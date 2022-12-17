use crate::util;
use crate::POOL;
use regex::Regex;

use serenity::model::channel::Message;
use serenity::client::Context;

pub async fn process_message(ctx: &Context, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "new message '{}' by {} in {}",
        msg.content.replace("\n", " "),
        msg.author.name,
        msg.channel_id
    );
    let re = Regex::new(r"> (.*)").unwrap();
    let maybe_match = re.find(&msg.content);

    let pool = POOL.get().unwrap();

    if let None = maybe_match {
        // nothing to do, message is not a quote
        return Ok(());
    }
    let query_res = sqlx::query!(
        "SELECT id FROM quotes WHERE content = $1 AND author_id = $2",
        &util::remove_my_deadname(&msg.content),
        msg.author.id.0.to_string(),
    )
    .fetch_optional(pool)
    .await?;

    if query_res.is_some() {
        // nothing to do, quote exists
        return Ok(());
    }

    println!("found a new quote 🎉: {}", &msg.content.replace("\n", ""));

    // let chrono_timestamp = chrono::DateTime::from_utc(NaiveDateTime::from_timestamp(, nsecs));
    sqlx::query!(
            "INSERT INTO quotes (content, author_id, sent_at, avatar_url, message_id, channel_id) VALUES ($1, $2, $3, $4, $5, $6)",
            util::remove_my_deadname(&msg.content),
            msg.author.id.0.to_string(),
            *msg.timestamp,
            msg.author.avatar_url(),
            msg.id.0.to_string(),
            msg.channel(&ctx).await.unwrap().id().0.to_string()
        )
        .execute(pool)
        .await?;

    Ok(())
}
