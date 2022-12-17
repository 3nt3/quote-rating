use std::fs;

use crate::models;

/// Removes all occurences of deadname and Deadname and replaces them with [Nia]
pub fn remove_my_deadname(text: &str) -> String {
    let deadname = get_config().unwrap().deadname;
    text.replace(&deadname, "[Nia]")
        .replace(&some_kind_of_uppercase_first_letter(&deadname), "[Nia]")
}

pub fn some_kind_of_uppercase_first_letter(s: &str) -> String {
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
