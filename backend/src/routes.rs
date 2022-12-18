use crate::{models, POOL};

use std::collections::HashMap;

use crate::discord;
use chrono::Utc;
use rocket::{
    futures::TryFutureExt,
    serde::{json::Json, Serialize},
    State,
};
use serenity::{
    model::prelude::{ChannelId, GuildId, MessageId, UserId},
    Client,
};

#[get("/quote?<prefer_unrated>&<only_images>")]
pub async fn get_quote(
    client: &State<Client>,
    prefer_unrated: bool,
    only_images: bool,
) -> Json<Vec<models::Quote>> {
    let pool = POOL.get().unwrap();

    struct QuoteButNotReally {
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

    let quote_records = 
        (match (only_images, prefer_unrated) {
            (true, true) => sqlx::query_as!(QuoteButNotReally, r#"
                select quotes.*, coalesce(sum(v.vote), 0::BIGINT) as score
                from quotes
                         left join votes v on quotes.id = v.quote_id
                where quotes.image_url is not null
                group by quotes.id
                order by (select (count(1) + (random() * 0.01)) from votes where votes.quote_id = id) asc
                limit 2
            "#).fetch_all(pool).await.unwrap(),
            (true, false) => 
                sqlx::query_as!(QuoteButNotReally, "SELECT quotes.*, coalesce(SUM(v.vote), 0::BIGINT) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id where quotes.image_url is not null GROUP BY quotes.id ORDER BY random()  LIMIT 2")
                .fetch_all(pool).await.unwrap(),
            (false, true) => sqlx::query_as!(QuoteButNotReally, r#"select quotes.*, coalesce(sum(v.vote), 0::BIGINT) as score
                                        from quotes
                                                 left join votes v on quotes.id = v.quote_id
                                        group by quotes.id
                                        order by (select (count(1) + (random() * 0.01)) from votes where votes.quote_id = id) asc
                                        limit 2
                                        "#).fetch_all(pool).await.unwrap(),
            (false, false) => 
                sqlx::query_as!(QuoteButNotReally, "SELECT quotes.*, coalesce(SUM(v.vote), 0::BIGINT) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id ORDER BY random() LIMIT 2")
            .fetch_all(pool).await.unwrap(),
        });

    // I kind of hate how this is structured
    let (r1, r2) = (&quote_records[0], &quote_records[1]);
    let (u1, u2) = (
        discord::get_username(&client, u64::from_str_radix(&r1.author_id, 10).unwrap())
            .await
            .unwrap_or((&"user not found").to_string()),
        discord::get_username(&client, u64::from_str_radix(&r2.author_id, 10).unwrap())
            .await
            .unwrap_or((&"user not found").to_string()),
    );

    discord::replace_mentions(&client, "".to_string()).await;

    let q1 = models::Quote {
        id: r1.id,
        content: discord::replace_mentions(&client, r1.content.clone()).await,
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
        image_url: r1.image_url.clone(),
    };
    let q2 = models::Quote {
        id: r2.id,
        content: discord::replace_mentions(&client, r2.content.clone()).await,
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
        image_url: r2.image_url.clone(),
    };

    Json(vec![q1, q2])
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
