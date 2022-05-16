#[macro_use]
extern crate rocket;

use chrono::{serde::ts_milliseconds, Utc};
use once_cell::sync::OnceCell;
use rocket::{
    futures::TryFutureExt,
    serde::{json::Json, Serialize},
};
use sqlx::{postgres::PgPoolOptions, query, Pool, Postgres};
use std::env;

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
    score: i64,
}

#[get("/quote")]
async fn get_quote() -> Json<Quote> {
    let pool = POOL.get().unwrap();
    let result = sqlx::query!("SELECT quotes.*, SUM(v.vote) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id ORDER BY random() LIMIT 1")
        .fetch_one(pool)
        .map_ok(|r| {
            Json(Quote {
                id: r.id,
                content: r.content,
                author_id: u64::from_str_radix(&r.author_id, 10).unwrap(),
                created_at: chrono::DateTime::<Utc>::from_utc(r.created_at.unwrap(), Utc),
                sent_at: chrono::DateTime::<Utc>::from_utc(r.sent_at.unwrap(), Utc),
                score: r.score.unwrap_or(0)
            })
        })
        .await
        .unwrap();

    result
}

#[post("/vote/<id>/<vote>")]
async fn vote(id: i32, vote: i32) -> Json<Quote> {
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
    .map_ok(|r| {
        Json(Quote {
            id: r.id,
            content: r.content,
            author_id: u64::from_str_radix(&r.author_id, 10).unwrap(),
            created_at: chrono::DateTime::<Utc>::from_utc(r.created_at.unwrap(), Utc),
            sent_at: chrono::DateTime::<Utc>::from_utc(r.sent_at.unwrap(), Utc),
            score: r.score.unwrap_or(0),
        })
    })
    .await
    .unwrap();

    result
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    POOL.set(pool).unwrap();

    rocket::build().mount("/", routes![get_quote, vote])
}
