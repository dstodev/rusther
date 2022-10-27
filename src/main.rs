#![crate_name = "rusther"]

use std::env;
use std::fs;

use log::LevelFilter;
use serenity::prelude::*;
use simple_logger::SimpleLogger;
use tokio::runtime::Handle;

use commands::ConnectFourDiscord;
use rusther::Arbiter;

mod commands;
mod rusther;
mod utility;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), String> {
    SimpleLogger::new()
        .with_colors(true)
        .with_local_timestamps()
        .with_level(LevelFilter::Off)
        .env() // Must appear after .with_level() to take effect; enables RUST_LOG environment var
        .with_module_level("rusther", LevelFilter::Debug) // But this line takes ultimate precedence for module-level logging
        .init()
        .unwrap();

    log::info!("Logger initialized");
    log::debug!("  With debug messages");
    log::trace!("  With trace messages");

    let mut arbiter = Arbiter::new(Handle::current());

    arbiter.register_event_handler(commands::Ping::new())?;
    arbiter.register_event_handler(commands::Announce)?;
    arbiter.register_event_handler(ConnectFourDiscord::new())?;

    let token = get_token().unwrap();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(arbiter)
        .await
        .expect("Could not create client!");

    if let Err(reason) = client.start().await {
        log::debug!("Client failed to start because {:?}", reason);
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
