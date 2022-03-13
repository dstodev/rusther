use std::env;
use std::fs;

use serenity::{
    prelude::*,
};

use rusther::Arbiter;

mod rusther;
mod commands;

#[tokio::main]
async fn main() -> Result<(), String> {
    let token = get_token().unwrap();

    let mut arbiter = Arbiter::new();

    arbiter.register_text_command("ping", Box::new(commands::Ping::new()))?;

    let mut client = Client::builder(token)
        .event_handler(arbiter)
        .await
        .expect("Could not create client!");

    if let Err(reason) = client.start().await {
        println!("Client failed with: {:?}", reason);
    }

    Ok(())
}

fn get_token() -> Result<String, &'static str> {
    if let Ok(t) = env::var("DISCORD_SERVER_TOKEN") {
        return Ok(t);
    } else if let Ok(t) = fs::read_to_string("secret") {
        return Ok(String::from(t.trim()));
    }
    Err("Could not find server token in environment \
         variable 'DISCORD_SERVER_TOKEN' or file 'secret'")
}
