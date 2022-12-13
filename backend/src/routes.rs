use crate::{models, POOL};

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

#[get("/quote?<prefer_unrated>")]
pub async fn get_quote(client: &State<Client>, prefer_unrated: bool) -> Json<Vec<models::Quote>> {
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
    }

    let quote_records;
    if prefer_unrated {
        quote_records = sqlx::query_as!(QuoteButNotReally, r#"select quotes.*, coalesce(sum(v.vote), 0::BIGINT) as score
                                        from quotes
                                                 left join votes v on quotes.id = v.quote_id
                                        group by quotes.id
                                        order by (select (count(1) + (random() * 0.01)) from votes where votes.quote_id = id) asc
                                        limit 2
                                        "#).fetch_all(pool).await.unwrap();
    } else {
        quote_records = sqlx::query_as!(QuoteButNotReally, "SELECT quotes.*, coalesce(SUM(v.vote), 0::BIGINT) AS score FROM quotes LEFT JOIN votes v on quotes.id = v.quote_id GROUP BY quotes.id ORDER BY random() LIMIT 2")
        .fetch_all(pool).await.unwrap();
    }

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

    let q1 = models::Quote {
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
    let q2 = models::Quote {
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
