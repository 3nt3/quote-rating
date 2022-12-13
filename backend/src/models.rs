use chrono::{serde::ts_milliseconds, Utc};
use rocket::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Quote {
    pub id: i32,
    pub content: String,
    pub author_id: u64,
    #[serde(with = "ts_milliseconds")]
    pub created_at: chrono::DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    pub sent_at: chrono::DateTime<Utc>,
    pub avatar_url: String,
    pub username: String,
    pub score: i64,
    pub channel_id: u64,
    pub message_id: u64,
    pub message_link: String,
}
