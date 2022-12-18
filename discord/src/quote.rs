use crate::util;
use crate::POOL;
use regex::Regex;

use serenity::client::Context;
use serenity::model::channel::Message;

pub struct QuoteInfo {
    pub is_quote: bool,
    pub is_duplicate: bool,
}

pub async fn process_message(
    ctx: &Context,
    msg: Message,
    silent: bool,
) -> Result<QuoteInfo, Box<dyn std::error::Error>> {
    let re = Regex::new(r"> (.*)").unwrap();
    let maybe_match = re.find(&msg.content);

    let pool = POOL.get().unwrap();

    if let None = maybe_match {
        // nothing to do, message is not a quote
        return Ok(QuoteInfo {
            is_quote: false,
            is_duplicate: false,
        });
    }

    let images: Vec<String> = msg
        .attachments
        .iter()
        .filter(|a| {
            (&(a.content_type.as_ref()))
                .unwrap_or(&"".to_string())
                .starts_with("image")
        })
        .map(|a| a.url.to_owned())
        .collect();
    for image in &images {
        println!("an image!! {:?}", image);
    }

    let query_res = sqlx::query!(
        "SELECT id FROM quotes WHERE content = $1",
        util::remove_my_deadname(&msg.content),
    )
    .fetch_optional(pool)
    .await?;

    if query_res.is_some() {
        // nothing to do, quote exists
        return Ok(QuoteInfo {
            is_quote: true,
            is_duplicate: true,
        });
    }

    if !silent {
        println!("found a new quote 🎉: {}", &msg.content.replace("\n", ""));
    }

    // let chrono_timestamp = chrono::DateTime::from_utc(NaiveDateTime::from_timestamp(, nsecs));
    sqlx::query!(
            "INSERT INTO quotes (content, author_id, sent_at, avatar_url, message_id, channel_id, image_url) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            util::remove_my_deadname(&msg.content),
            msg.author.id.0.to_string(),
            *msg.timestamp,
            msg.author.avatar_url(),
            msg.id.0.to_string(),
            msg.channel(&ctx).await.unwrap().id().0.to_string(),
            images.get(0)
        )
        .execute(pool)
        .await?;

    Ok(QuoteInfo {
        is_quote: true,
        is_duplicate: false,
    })
}
