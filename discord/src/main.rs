use std::{fs, process::exit};

use regex::Regex;
use serde::Deserialize;
use serenity::{
    async_trait,
    framework::{
        standard::{
            macros::{command, group},
            CommandResult,
        },
        StandardFramework,
    },
    futures::StreamExt,
    model::prelude::*,
    prelude::*,
    Client,
};
use toml::Value;

// struct Handler;
// impl EventHandler for Handler {
//     fn message(&self, context: Context, msg: Message) {
//         unimplemented!();
//     }
// }

#[derive(Deserialize, Debug)]
struct Config {
    token: String,
    channel_id: u64,
}

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, data_about_bot: Ready) {
        let config_result = get_config();
        if let None = config_result {
            eprintln!("error reading config.");
            return;
        }
        let config = config_result.unwrap();

        let re = Regex::new(r"> (.*)").unwrap();

        let mut messages = ChannelId(config.channel_id).messages_iter(&context).boxed();
        let mut n: i64 = 0;
        let mut total: i64 = 0;

        while let Some(message_result) = messages.next().await {
            total += 1;
            match message_result {
                Ok(message) => {
                    let mat = re.find(&message.content);
                    if let Some(_) = mat {
                        n += 1;
                    }
                }
                Err(error) => eprintln!("error getting messages: {}", error),
            }
        }

        println!("{n} quotes found in {total} messages");
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

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
