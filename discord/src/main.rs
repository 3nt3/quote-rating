use std::{env, fs, process::exit};

use chrono::{NaiveDateTime, Utc};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::Deserialize;
use serenity::{
    async_trait,
    framework::{standard::macros::group, StandardFramework},
    futures::StreamExt,
    model::prelude::*,
    prelude::*,
    Client,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
// use warp::Filter;

#[derive(Deserialize, Debug)]
struct Config {
    token: String,
    channel_id: u64,
}

#[group]
struct General;

struct Handler;

static POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _: Context, msg: Message) {
        println!(
            "new message '{}' by {} in {}",
            msg.content.replace("\n", " "),
            msg.author.name,
            msg.channel_id
        );
        let re = Regex::new(r"> (.*)").unwrap();
        let mat = re.find(&msg.content);

        let pool = POOL.get().unwrap();

        if let None = mat {
            return;
        }
        let query_res = sqlx::query!(
            "SELECT id FROM quotes WHERE content = $1 AND author_id = $2",
            &msg.content,
            msg.author.id.0.to_string(),
        )
        .fetch_optional(pool)
        .await;
        match query_res {
            Ok(foo) => {
                if let Some(_) = foo {
                    return;
                }
            }
            Err(error) => {
                eprintln!("error querying db: {}", error);
                return;
            }
        }

        println!("found a new quote 🎉: {}", &msg.content.replace("\n", ""));

        // let chrono_timestamp = chrono::DateTime::from_utc(NaiveDateTime::from_timestamp(, nsecs));
        let insert_res: sqlx::Result<sqlx::postgres::PgQueryResult> = sqlx::query!(
            "INSERT INTO quotes (content, author_id, sent_at) VALUES ($1, $2, $3)",
            &msg.content,
            msg.author.id.0.to_string(),
            *msg.timestamp
        )
        .execute(pool)
        .await;

        if let Err(query_error) = insert_res {
            eprintln!("error adding to db: {}", query_error);
        }
    }

    async fn ready(&self, context: Context, _: Ready) {
        let config_result = get_config();
        if let None = config_result {
            eprintln!("error reading config.");
            return;
        }
        let config = config_result.unwrap();

        let re = Regex::new(r"> (.*)").unwrap();

        let mut messages = ChannelId(config.channel_id).messages_iter(&context).boxed();
        let mut n_quotes: i64 = 0;
        let mut n_new_quotes: i64 = 0;
        let mut total: i64 = 0;

        let pool = POOL.get().unwrap();
        while let Some(message_result) = messages.next().await {
            total += 1;
            match message_result {
                Ok(message) => {
                    let mat = re.find(&message.content);

                    if let Some(_) = mat {
                        n_quotes += 1;
                        let query_res = sqlx::query!(
                            "SELECT id FROM quotes WHERE content = $1 AND author_id = $2",
                            &message.content,
                            message.author.id.0.to_string(),
                        )
                        .fetch_optional(pool)
                        .await;
                        match query_res {
                            Ok(foo) => {
                                if let Some(_) = foo {
                                    // if there is a match for content and author,
                                    // do not insert a duplicate and continue
                                    // to the next message
                                    continue;
                                }
                            }
                            Err(error) => {
                                eprintln!("error querying db: {}", error);
                                continue;
                            }
                        }

                        n_new_quotes += 1;

                        println!("found a new quote 🎉: {}", &message.content);

                        // message.timestamp.with_timezone(chrono::Utc)
                        let insert_res: sqlx::Result<sqlx::postgres::PgQueryResult> = sqlx::query!(
                            "INSERT INTO quotes (content, author_id, sent_at) VALUES ($1, $2, $3)",
                            &message.content,
                            message.author.id.0.to_string(),
                            *message.timestamp, // the dereference converts serenity::Timestamp to
                                                // chrono::DateTime
                        )
                        .execute(pool)
                        .await;

                        if let Err(query_error) = insert_res {
                            eprintln!("error adding to db: {}", query_error);
                        }
                    }
                }
                Err(error) => eprintln!("error getting messages: {}", error),
            }
        }

        println!("{n_new_quotes} new quotes ({n_quotes} total) found in {total} messages");
    }
}

#[tokio::main]
async fn main() {
    // read configuration
    let maybe_config = get_config();
    if let None = maybe_config {
        eprintln!("couldn't read config");
        exit(1);
    }
    let config = maybe_config.unwrap();

    let pool = connect_db().await;
    POOL.set(pool).unwrap();

    // set up discord bot
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(config.token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
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

async fn connect_db() -> Pool<Postgres> {
    dotenv::dotenv().ok();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}
