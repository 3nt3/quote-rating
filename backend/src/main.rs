#![feature(async_closure)]
#[macro_use]
extern crate rocket;

mod discord;
mod models;
mod routes;

use chrono::{serde::ts_milliseconds, Utc};
use once_cell::sync::OnceCell;
use rocket::{
    futures::TryFutureExt,
    serde::{json::Json, Serialize},
    State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use serde::Deserialize;
use serenity::model::prelude::MessageId;

use serenity::model::prelude::ChannelId;
use serenity::model::prelude::{GuildId, UserId};
use serenity::prelude::*;

use crate::discord::get_username;
use bigdecimal::ToPrimitive;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use std::{collections::HashMap, fs};

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

#[get("/leaderboard")]
async fn get_leaderboard(client: &State<Client>) -> Json<Vec<models::Quote>> {
    let pool = POOL.get().unwrap();
    let res = sqlx::query!(
        "SELECT * FROM (SELECT quotes.*, SUM(v.vote) AS score
        FROM quotes
                 LEFT JOIN votes v on quotes.id = v.quote_id
        GROUP By quotes.id) AS x
        WHERE score is not null
        ORDER BY score DESC
        LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .unwrap();

    let username_futures = res
        .iter()
        .map(|r| get_username(&client, u64::from_str_radix(&r.author_id, 10).unwrap()));

    // TODO: make this less slow
    let username_results = futures::future::join_all(username_futures).await;

    let message_links_futures = res.iter().map(|r| async {
        let message_id = MessageId(u64::from_str_radix(&r.message_id, 10).unwrap());
        let link = message_id.link_ensured(
            &client.cache_and_http,
            ChannelId(u64::from_str_radix(&r.channel_id, 10).unwrap()),
            Some(GuildId(816943824630710272)),
        );
        return link.await.clone();
    });
    let message_links = futures::future::join_all(message_links_futures).await;

    let content_futures = res
        .iter()
        .map(|r| discord::replace_mentions(client, r.content.clone()));
    let content_results = futures::future::join_all(content_futures).await;


    let items = res
        .iter()
        .enumerate()
        .map(move |(i, r)| models::Quote {
            id: r.id,
            content: content_results.get(i).unwrap_or(&r.content).clone(),
            author_id: u64::from_str_radix(&r.author_id, 10).unwrap(),
            avatar_url: r.avatar_url.clone(),
            username: username_results[i]
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_string(),
            created_at: r.created_at,
            sent_at: r.sent_at,
            score: r.score.unwrap_or(0),
            channel_id: u64::from_str_radix(&r.channel_id, 10).unwrap(),
            message_id: u64::from_str_radix(&r.message_id, 10).unwrap(),
            message_link: (&message_links.get(i).unwrap_or(&"".to_string()))
                .to_string()
                .to_string(),
            image_url: r.image_url.clone(),
        })
        .collect::<Vec<models::Quote>>();

    Json(items)
}

#[derive(Debug, Serialize)]
struct PersonWithNumber {
    username: Option<String>,
    user_id: String,
    sum_score: Option<f64>,
    avg_score: Option<f64>,
    n_votes: Option<i64>,
    n_quotes: Option<i64>,
}

#[get("/funniest-people")]
async fn funniest_people(client: &State<Client>) -> Json<Vec<PersonWithNumber>> {
    let pool = POOL.get().unwrap();

    let res = sqlx::query!("select q.author_id, sum(x.score) as sum_score, avg(x.score) as avg_score, count(x.score) as n_votes from (select quote_id, sum(vote) as score from votes left join quotes q on votes.quote_id = q.id where author_id is not null group by quote_id) as x left join quotes as q on quote_id = q.id group by q.author_id order by sum_score desc ;").fetch_all(pool).await.unwrap();

    let username_futures = res
        .iter()
        .map(|r| get_username(&client, u64::from_str_radix(&r.author_id, 10).unwrap()));
    let usernames = futures::future::join_all(username_futures).await;

    let n_quotes_vec = sqlx::query!("select quotes.author_id, count(*) as n_quotes from quotes group by quotes.author_id order by n_quotes desc").fetch_all(pool).await.unwrap();
    let n_quotes: HashMap<String, i64> = n_quotes_vec
        .iter()
        .map(|x| (x.author_id.clone(), x.n_quotes.unwrap_or(0)))
        .collect();

    Json(
        res.iter()
            .enumerate()
            .map(move |(i, r)| PersonWithNumber {
                username: usernames[i].as_ref().cloned(),
                user_id: (&r.author_id).to_string(),
                sum_score: r.sum_score.as_ref().map(|x| x.to_f64().unwrap_or(0.0)),
                avg_score: r.avg_score.as_ref().map(|x| x.to_f64().unwrap_or(0.0)),
                n_votes: r.n_votes,
                n_quotes: n_quotes.get(&r.author_id).copied(),
            })
            .collect(),
    )
}

#[post("/vote/<id>/<vote>")]
async fn vote(client: &State<Client>, id: i32, vote: i32) -> Json<models::Quote> {
    let pool = POOL.get().unwrap();

    let vote_cleaned = if vote > 0 {
        1
    } else if vote < 0 {
        -1
    } else {
        0
    };

    let insert_res = sqlx::query!(
        "INSERT INTO votes (quote_id, vote) VALUES ($1, $2)",
        id,
        vote_cleaned
    )
    .execute(pool)
    .await;

    if let Err(err) = insert_res {
        eprintln!("error inserting: {}", err);
    }

    // no one every wrote more conscise sql
    // who would use two queries if you could have one no one understands
    let result = sqlx::query!(
        "SELECT * FROM (SELECT quotes.*, SUM(v.vote) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id) AS x WHERE x.id = $1",
        id
    )
    .fetch_one(pool)
    .map_ok(async move |r| {
        Json(models::Quote {
            id: r.id,
            content: r.content,
            author_id: u64::from_str_radix(&r.author_id, 10).unwrap(),
            created_at: r.created_at,
            username: get_username(&client, u64::from_str_radix(&r.author_id, 10).unwrap()).await.unwrap_or(r.author_id),
            avatar_url: r.avatar_url,
            sent_at: r.sent_at,
            score: r.score.unwrap_or(0),
            channel_id: u64::from_str_radix(&r.channel_id, 10).unwrap(),
            message_id: u64::from_str_radix(&r.message_id, 10).unwrap(),
            message_link: MessageId(u64::from_str_radix(&r.message_id, 10).unwrap())
                .link_ensured(
                    &client.cache_and_http,
                    ChannelId(u64::from_str_radix(&r.channel_id, 10).unwrap()),
                    Some(GuildId(816943824630710272)),
                )
                .await,
            image_url: r.image_url.clone(),
        })
    })
    .await
    .unwrap();

    result.await
}

/// Tries to figure out a channel for a message id
// async fn find_channel(client: &Client, message_id: u64) -> Result<u64, serenity::Error> {
//     let channels = client
//         .cache_and_http
//         .http
//         .get_channels(816943824630710272)
//         .await?;
//
//     let channel_id: Option<u64> = None;
//     for channel in channels {
//         let maybe_msg = client
//             .cache_and_http
//             .http
//             .get_message(channel.id.0, message_id)
//             .await;
//         if let Ok(_) = maybe_msg {
//             return Ok(channel.id.0);
//         }
//     }
//
//     Err(serenity::Error::Other("not found"))
// }

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    // read configuration
    let maybe_config = get_config();
    if let None = maybe_config {
        eprintln!("couldn't read config");
        std::process::exit(1);
    }
    let config = maybe_config.unwrap();

    // set up discord bot

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES;

    let client = Client::builder(config.token, intents)
        .await
        .expect("Error creating client");

    // thread::spawn(|| {
    //     let mut rt = Runtime::new().unwrap();
    //     rt.block_on(async move {
    //         client.start().await;
    //     });
    // });

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    POOL.set(pool).unwrap();

    let allowed_origins =
        AllowedOrigins::some_exact(&["http://localhost:5173", "https://quotess.3nt3.de"]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_headers: AllowedHeaders::some(&[]),
        allow_credentials: false,
        ..Default::default()
    }
    .to_cors()?;

    let _ = rocket::build()
        .manage(client)
        .mount(
            "/",
            routes![
                routes::get_quote,
                routes::get_all_scores,
                vote,
                get_leaderboard,
                routes::get_stats,
                funniest_people
            ],
        )
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}

#[derive(Serialize)]
struct Stats {
    num_quotes: i64,
    num_votes: i64,
    num_rated: i64,
}

#[derive(Debug, PartialEq, FromFormField)]
enum Format {
    Prometheus,
    Json,
}


#[derive(Deserialize)]
struct Config {
    token: String,
    guild_id: u64,
}

fn get_config() -> Option<Config> {
    let content_result = fs::read_to_string("./config.toml");
    if let Err(error) = content_result {
        println!("error reading config.toml: {error}");
        return None;
    }
    let content = content_result.unwrap();

    match toml::from_str(&content) {
        Ok(config) => {
            return Some(config);
        }
        Err(error) => {
            eprintln!("{}", error);
            return None;
        }
    }
}
