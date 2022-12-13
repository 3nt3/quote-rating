use serenity::{
    model::prelude::{GuildId, UserId},
    Client,
};

use crate::POOL;

pub async fn get_username(client: &Client, user_id: u64) -> Option<String> {
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
