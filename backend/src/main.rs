#![feature(async_closure)]
#[macro_use]
extern crate rocket;

use chrono::{serde::ts_milliseconds, Utc};
use futures::{stream, Future, StreamExt};
use once_cell::sync::OnceCell;
use rocket::{
    futures::TryFutureExt,
    serde::{json::Json, Serialize},
    State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use serde::Deserialize;

use futures::future;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{
    async_trait, client,
    model::prelude::{GuildId, UserId},
};
use serenity::{
    client::bridge::gateway::ShardManager,
    model::prelude::{Message, MessageId},
};
use serenity::{http::Http, model::prelude::ChannelId};

use sqlx::{postgres::PgPoolOptions, query, Pool, Postgres};
use std::{collections::HashMap, fs, hash::Hash};
use std::{env, thread};

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Quote {
    id: i32,
    content: String,
    author_id: u64,
    #[serde(with = "ts_milliseconds")]
    created_at: chrono::DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    sent_at: chrono::DateTime<Utc>,
    avatar_url: String,
    username: String,
    score: i64,
    channel_id: u64,
    message_id: u64,
    message_link: String,
}

#[get("/quote")]
async fn get_quote(client: &State<Client>) -> Json<Vec<Quote>> {
    let pool = POOL.get().unwrap();
    let quote_records = sqlx::query!("SELECT quotes.*, SUM(v.vote) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id ORDER BY random() LIMIT 2")
        .fetch_all(pool).await.unwrap();

    // I kind of hate how this is structured
    let (r1, r2) = (&quote_records[0], &quote_records[1]);
    let (u1, u2) = (
        get_username(&client, u64::from_str_radix(&r1.author_id, 10).unwrap())
            .await
            .unwrap_or((&"user not found").to_string()),
        get_username(&client, u64::from_str_radix(&r2.author_id, 10).unwrap())
            .await
            .unwrap_or((&"user not found").to_string()),
    );

    let q1 = Quote {
        id: r1.id,
        content: r1.content.clone(),
        author_id: u64::from_str_radix(&r1.author_id, 10).unwrap(),
        avatar_url: (r1.avatar_url.clone()),
        username: u1,
        created_at: r1.created_at,
        sent_at: r1.sent_at,
        score: r1.score.unwrap_or(0),
        channel_id: u64::from_str_radix(&r1.channel_id, 10).unwrap(),
        message_id: u64::from_str_radix(&r1.message_id, 10).unwrap(),
        message_link: MessageId(u64::from_str_radix(&r1.message_id, 10).unwrap())
            .link_ensured(
                &client.cache_and_http,
                ChannelId(u64::from_str_radix(&r1.channel_id, 10).unwrap()),
                Some(GuildId(816943824630710272)),
            )
            .await,
    };
    let q2 = Quote {
        id: r2.id,
        content: r2.content.clone(),
        author_id: u64::from_str_radix(&r2.author_id, 10).unwrap(),
        avatar_url: (r2.avatar_url.clone()),
        username: u2,
        created_at: r2.created_at,
        sent_at: r2.sent_at,
        score: r2.score.unwrap_or(0),
        channel_id: u64::from_str_radix(&r2.channel_id, 10).unwrap(),
        message_id: u64::from_str_radix(&r2.message_id, 10).unwrap(),
        message_link: MessageId(u64::from_str_radix(&r2.message_id, 10).unwrap())
            .link_ensured(
                &client.cache_and_http,
                ChannelId(u64::from_str_radix(&r2.channel_id, 10).unwrap()),
                Some(GuildId(816943824630710272)),
            )
            .await,
    };

    Json(vec![q1, q2])
}

async fn get_username(client: &Client, user_id: u64) -> Option<String> {
    let pool = POOL.get().unwrap();
    let res = sqlx::query!(
        "SELECT username from username_cache WHERE user_id = $1",
        user_id.to_string()
    )
    .fetch_one(pool)
    .await;

    match res {
        Ok(r) => {
            return Some(r.username);
        }
        Err(_) => {
            let maybe_nick = GuildId(816943824630710272)
                .member(&client.cache_and_http.http, UserId(user_id))
                .await
                .map(|m| m.nick)
                .ok()
                .flatten();

            if let Some(ref nick) = maybe_nick {
                sqlx::query!(
                    "insert into username_cache (user_id, username) values ($1, $2)",
                    user_id.to_string(),
                    nick
                )
                .execute(pool)
                .await
                .unwrap();
            } else {
                let maybe_username = UserId(user_id)
                    .to_user(&client.cache_and_http)
                    .await
                    .map(|x| x.name)
                    .ok();

                if let Some(ref username) = maybe_username {
                    sqlx::query!(
                        "insert into username_cache (user_id, username) values ($1, $2)",
                        user_id.to_string(),
                        username
                    )
                    .execute(pool)
                    .await
                    .unwrap();
                }

                return maybe_username;
            }

            return maybe_nick;
        }
    }
}

#[get("/leaderboard")]
async fn get_leaderboard(client: &State<Client>) -> Json<Vec<Quote>> {
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

    let items = res
        .iter()
        .enumerate()
        .map(move |(i, r)| Quote {
            id: r.id,
            content: r.content.clone(),
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
        })
        .collect::<Vec<Quote>>();

    Json(items)
}

#[derive(Debug, Serialize)]
struct PersonWithNumber {
    username: Option<String>,
    user_id: String,
    score: Option<i64>,
}

#[get("/funniest-people")]
async fn funniest_people(client: &State<Client>) -> Json<Vec<PersonWithNumber>> {
    let pool = POOL.get().unwrap();

    let res = sqlx::query!("select * from (SELECT x.author_id, x.score -- this combines author_id and score
            FROM (SELECT quotes.*, SUM(v.vote) AS score -- this calculates score from single votes
                  FROM quotes
                           LEFT JOIN votes v on quotes.id = v.quote_id
                  GROUP By quotes.id) AS x
            WHERE score is not null
            group by x.author_id, x.score
            ORDER BY score DESC) as z left join username_cache on username_cache.user_id = z.author_id -- this uses username_cache").fetch_all(pool).await.unwrap();

    let username_futures = res.iter().map(|r| {
        get_username(
            &client,
            u64::from_str_radix(&r.user_id.as_ref().unwrap_or(&"0".to_string()), 10).unwrap(),
        )
    });
    let usernames = futures::future::join_all(username_futures).await;

    Json(
        res.iter()
            .enumerate()
            .map(move |(i, r)| PersonWithNumber {
                username: usernames[i].as_ref().cloned(),
                user_id: (&r.user_id.as_ref())
                    .unwrap_or(&"user id not found".to_string())
                    .to_string(),
                score: r.score,
            })
            .collect(),
    )
}

#[post("/vote/<id>/<vote>")]
async fn vote(client: &State<Client>, id: i32, vote: i32) -> Json<Quote> {
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
        Json(Quote {
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
        })
    })
    .await
    .unwrap();

    result.await
}

/// Tries to figure out a channel for a message id
async fn find_channel(client: &Client, message_id: u64) -> Result<u64, serenity::Error> {
    let channels = client
        .cache_and_http
        .http
        .get_channels(816943824630710272)
        .await?;

    let channel_id: Option<u64> = None;
    for channel in channels {
        let maybe_msg = client
            .cache_and_http
            .http
            .get_message(channel.id.0, message_id)
            .await;
        if let Ok(_) = maybe_msg {
            return Ok(channel.id.0);
        }
    }

    Err(serenity::Error::Other("not found"))
}

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

    let mut client = Client::builder(config.token, intents)
        .await
        .expect("Error creating client");

    // thread::spawn(|| {
    //     let mut rt = Runtime::new().unwrap();
    //     rt.block_on(async move {
    //         client.start().await;
    //     });
    // });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    POOL.set(pool).unwrap();

    let allowed_origins =
        AllowedOrigins::some_exact(&["http://localhost:5173", "https://quotes.3nt3.de"]);
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
            routes![get_quote, vote, get_leaderboard, get_stats, funniest_people],
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

#[get("/stats")]
async fn get_stats() -> String {
    let pool = POOL.get().unwrap();

    let num_quotes = sqlx::query!("SELECT count(id) FROM quotes")
        .fetch_one(pool)
        .map_ok(|r| r.count.unwrap())
        .await
        .unwrap();

    let num_votes = sqlx::query!("SELECT count(1) FROM votes")
        .fetch_one(pool)
        .map_ok(|r| r.count.unwrap())
        .await
        .unwrap();

    let num_rated = sqlx::query!(
        "select count(1) from (select 1 from votes left join quotes q on votes.quote_id = q.id group by quote_id) as _"
    )
    .fetch_one(pool)
    .map_ok(|r| r.count.unwrap())
    .await
    .unwrap();

    serde_prometheus::to_string(
        &Stats {
            num_quotes,
            num_votes,
            num_rated,
        },
        None,
        HashMap::new(),
    )
    .unwrap()
}

#[derive(Deserialize)]
struct Config {
    token: String,
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
