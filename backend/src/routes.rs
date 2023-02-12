use crate::{
    models::{self, Quote},
    POOL, get_config,
};

use std::{collections::HashMap, any::{self, Any}};

use crate::discord;
use chrono::Utc;
use rocket::{
    futures::TryFutureExt,
    response::{content, status},
    serde::{json::Json, Serialize},
    State, http::Status,
};
use serenity::{
    model::prelude::{ChannelId, GuildId, MessageId, UserId},
    Client,
};
use sqlx::query_builder::QueryBuilder;

#[get("/quote?<prefer_unrated>&<only_images>&<only_good>")]
pub async fn get_quote(
    client: &State<Client>,
    prefer_unrated: bool,
    only_images: bool,
    only_good: bool,
) -> Result<Json<Quote>, status::Custom<Json<HashMap<&str, String>>>> {
    let pool = POOL.get().unwrap();

    // This is just a typed representation of the raw row from the database
    #[derive(sqlx::FromRow, Debug)]
    struct QuoteRecord {
        id: i32,
        content: String,
        author_id: String,
        sent_at: chrono::DateTime<Utc>,
        avatar_url: String,
        message_id: String,
        channel_id: String,
        created_at: chrono::DateTime<Utc>,
        score: Option<i64>, // there is literally no way this would ever be None sqlx is just dumb
        image_url: Option<String>,
    }

    let mut query_builder = QueryBuilder::new("select quotes.*, coalesce(sum(v.vote), 0::BIGINT) as score from quotes left join votes v on quotes.id = v.quote_id ");

    if only_images {
        query_builder.push("where quotes.image_url is not null ");
    }

    query_builder.push("group by quotes.id ");

    if only_good && !prefer_unrated {
        // FIXME: this will run out when all quotes are rated bad
        // solution: sort it and use the top ones instead of using HAVING to filter the rows
        query_builder.push("having coalesce(sum(v.vote), 0::BIGINT) > 0 ");
    }

    if prefer_unrated {
        query_builder.push("order by (select (count(1) + (random() * 0.01)) from votes where votes.quote_id = id) asc ");
    } else {
        query_builder.push("order by random() ");
    }

    query_builder.push("limit 1");

    let quote_records = query_builder
        .build_query_as::<QuoteRecord>()
        .fetch_all(pool)
        .await;


    if let Err(why) = quote_records {
        let mut response = HashMap::new();
        response.insert("error", why.to_string());
        return Err(status::Custom(Status::InternalServerError, Json(response)));
    }
    let quote_records = quote_records.unwrap();

    let quote_record = quote_records.get(0);
    if quote_record.is_none() {
        let mut response = HashMap::new();
        response.insert("error", "No quotes found".to_string());
        return Err(status::Custom(Status::InternalServerError, Json(response)));
    }
    let quote_record = quote_record.unwrap();

    let user_id = quote_record.author_id.parse::<u64>();
    if let Err(why) = user_id {
        let mut response = HashMap::new();
        response.insert("error", why.to_string());
        return Err(status::Custom(Status::InternalServerError, Json(response)));
    }
    let user_id = user_id.unwrap();

    let message_id = quote_record.message_id.parse::<u64>();
    if let Err(why) = message_id {
        let mut response = HashMap::new();
        response.insert("error", why.to_string());
        return Err(status::Custom(Status::InternalServerError, Json(response)));
    }

    let channel_id = quote_record.channel_id.parse::<u64>();
    if let Err(why) = channel_id {
        let mut response = HashMap::new();
        response.insert("error", why.to_string());
        return Err(status::Custom(Status::InternalServerError, Json(response)));
    }

    let config = get_config().unwrap();

    let quote = Quote {
        id: quote_record.id,
        content: discord::replace_mentions(client, quote_record.content.clone()).await,
        author_id: user_id,
        sent_at: quote_record.sent_at,
        avatar_url: quote_record.avatar_url.clone(),
        message_id: message_id.unwrap(),
        channel_id: channel_id.unwrap(),
        created_at: quote_record.created_at,
        score: quote_record.score.unwrap_or(0),
        image_url: quote_record.image_url.clone(),
        message_link: MessageId(quote_record.message_id.parse().unwrap())
            .link(ChannelId(quote_record.channel_id.parse().unwrap()), Some(GuildId(config.guild_id))),
        username: discord::get_username(client, user_id).await.unwrap_or("Unknown".to_string()),
    };

    Ok(Json(quote))
    // Json(vec![q1, q2])
}

#[derive(Serialize)]
pub struct Stats {
    num_quotes: i64,
    num_votes: i64,
    num_rated: i64,
}

#[derive(Debug, PartialEq, FromFormField)]
pub enum Format {
    Prometheus,
    Json,
}

#[get("/stats?<format>")]
pub async fn get_stats(format: Option<Format>) -> String {
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
        "select count(1) from (select 1 from votes inner join quotes q on votes.quote_id = q.id group by quote_id) as _"
    )
    .fetch_one(pool)
    .map_ok(|r| r.count.unwrap())
    .await
    .unwrap();

    dbg!(&format);

    match format.unwrap_or(Format::Prometheus) {
        Format::Prometheus => {
            return serde_prometheus::to_string(
                &Stats {
                    num_quotes,
                    num_votes,
                    num_rated,
                },
                None,
                HashMap::new(),
            )
            .unwrap();
        }
        Format::Json => {
            return serde_json::to_string(&Stats {
                num_quotes,
                num_votes,
                num_rated,
            })
            .unwrap();
        }
    }
}

#[get("/all-scores")]
pub async fn get_all_scores() -> Json<Vec<i64>> {
    struct Row {
        score: Option<i64>,
    }

    let pool = POOL.get().unwrap();
    let rows = sqlx::query_as!(Row, "select sum(v.vote) as score from quotes left join votes v on quotes.id = v.quote_id where v.quote_id is not null group by v.quote_id")
        .fetch_all(pool)
        .await.unwrap();

    let values: Vec<i64> = rows
        .iter()
        .map(|r| r.score)
        .filter(|x| x.is_some())
        .map(|x| x.unwrap_or(0))
        .collect();
    Json(values)
}
