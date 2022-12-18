use std::fs;

use crate::models;
use crate::{POOL,CONFIG};

use prompts::{confirm::ConfirmPrompt, Prompt};

/// Removes all occurences of deadname and Deadname and replaces them with [Nia]
pub fn remove_my_deadname(text: &str) -> String {
    let deadname = &CONFIG.get().unwrap().deadname;
    text.replace(&*deadname, "[Nia]")
        .replace(&some_kind_of_uppercase_first_letter(&deadname), "[Nia]")
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Reads ./config.toml and parses it
pub fn get_config() -> Option<models::Config> {
    // TODO: don't re-read the config every time it is accessed, should rather be stored somewhere
    // globally?
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

/// Removes duplicate quotes
pub async fn remove_duplicates() -> sqlx::Result<()> {
    let pool = POOL.get().unwrap();

    #[derive(Debug)]
    struct Row {
        id: i32,
    }

    let duplicates: Vec<Row> = sqlx::query_as!(Row,
        "select quotes.id from quotes right join (select quotes.content, count(*) from quotes group by quotes.content having count(*) > 1) as x on quotes.content = x.content")
    .fetch_all(pool)
    .await?;

    for duplicate in &duplicates {
        println!("{:?}", duplicate);
    }
    println!("Found {} duplicates.", duplicates.len());

    let mut prompt = ConfirmPrompt::new("Do you want to delete them?").set_initial(false);

    if duplicates.len() == 0 {
        return Ok(());
    }

    if let Ok(value) = prompt.run().await {
        if !value.unwrap_or(false) {
            return Ok(());
        }
        let comma_seperated = duplicates
            .iter()
            .map(|x| x.id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        sqlx::query(&format!(
            "delete from quotes where quotes.id in ({})",
            comma_seperated
        ))
        .execute(pool)
        .await?;

        println!("deleted {} duplicates", duplicates.len());
    }

    Ok(())
}
