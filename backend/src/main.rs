#![feature(async_closure)]
#[macro_use]
extern crate rocket;

use chrono::{serde::ts_milliseconds, Utc};
use once_cell::sync::OnceCell;
use rocket::{
    futures::TryFutureExt,
    serde::{json::Json, Serialize},
    State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use serde::Deserialize;

use serenity::client::bridge::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{
    async_trait, client,
    model::prelude::{GuildId, UserId},
};
use tokio::runtime::Runtime;

use sqlx::{postgres::PgPoolOptions, query, Pool, Postgres};
use std::fs;
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
    avatar_url: Option<String>,
    username: String,
    score: i64,
}

#[get("/quote")]
async fn get_quote(client: &State<Client>) -> Json<Quote> {
    let pool = POOL.get().unwrap();
    let result = sqlx::query!("SELECT quotes.*, SUM(v.vote) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id ORDER BY random() LIMIT 1")
        .fetch_one(pool)
        .map_ok(async move |r| {
            Json(Quote {
                id: r.id,
                content: r.content,
                author_id: u64::from_str_radix(&r.author_id, 10).unwrap(),
                avatar_url: r.avatar_url,
                username: get_username(&client, u64::from_str_radix(&r.author_id, 10).unwrap()).await.unwrap(),
                created_at: r.created_at,
                sent_at: r.sent_at,
                score: r.score.unwrap_or(0)
            })
        })
        .await
        .unwrap();

    result.await
}

async fn get_username(client: &Client, user_id: u64) -> Option<String> {
    GuildId(816943824630710272)
        .members(&client.cache_and_http.http, Some(1), UserId(user_id))
        .await
        .unwrap()
        .first()
        .unwrap()
        .nick
        .clone()
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
            username: get_username(&client, u64::from_str_radix(&r.author_id, 10).unwrap()).await.unwrap(),
            avatar_url: r.avatar_url,
            sent_at: r.sent_at,
            score: r.score.unwrap_or(0),
        })
    })
    .await
    .unwrap();

    result.await
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

    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_headers: AllowedHeaders::some(&[]),
        allow_credentials: false,
        ..Default::default()
    }
    .to_cors()?;

    let _ = rocket::build()
        .manage(client)
        .mount("/", routes![get_quote, vote])
        .attach(cors)
        .launch()
        .await?;

    Ok(())
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
