use std::collections::HashMap;

use serenity::{
    model::prelude::{GuildId, UserId},
    Client,
};

use regex::{Captures, Regex};

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

pub async fn replace_mentions(client: &Client, content: String) -> String {
    let mention_regex = Regex::new("<@!*&*([0-9]+)>").unwrap();

    if mention_regex.find(&content).is_none() {
        return content;
    }

    let captures_iter = mention_regex.captures_iter(&content);
    let mut usernames: HashMap<String, Option<String>> = HashMap::new();

    let mut username_futures = vec![];
    let mut user_ids: Vec<u64> = vec![];
    for captures in captures_iter {
        let user_id_str = captures.get(1);
        if user_id_str.is_none() {
            continue;
        }
        let user_id = u64::from_str_radix(user_id_str.unwrap().as_str(), 10).unwrap();

        if user_ids.contains(&user_id) {
            continue;
        }
        &username_futures.push(get_username(client, user_id));
        &user_ids.push(user_id);
    }

    for (i, username) in futures::future::join_all(username_futures)
        .await
        .iter()
        .enumerate()
    {
        let user_id = user_ids.get(i);
        if user_id.is_none() || username.is_none() {
            continue;
        }

        usernames.insert(user_id.unwrap().to_string(), username.clone());
    }

    mention_regex.replace_all(&content, |caps: &Captures| {
        let maybe_user_id = caps.get(1);
        match maybe_user_id {
            Some(user_id_match) => {
                let username = usernames
                    .get(user_id_match.as_str())
                    .unwrap_or(&None)
                    .clone();
                format!("<@{}>", &username.unwrap_or("user not found".to_string()))
            }
            None => "<@user not found>".to_string(),
        }
    })
    .to_string()
}
