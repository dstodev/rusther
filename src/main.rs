#![crate_name = "rusther"]

use std::{env, fs, path};

use log::LevelFilter;
use serenity::prelude::*;
use simple_logger::SimpleLogger;
use tokio::runtime::Handle;

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

    let arbiter = Arbiter::new(Handle::current()).with_all_commands();
    let token = get_token().unwrap();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(arbiter)
        .cache_settings(move |cache| cache.max_messages(100))
        .await
        .expect("Could not create client!");

    if let Err(reason) = client.start_autosharded().await {
        log::debug!("Client failed to start because {:?}", reason);
    }

    Ok(())
}

fn get_token() -> Result<String, String> {
    const ENV_VAR: &'static str = "DISCORD_SERVER_TOKEN";
    const SECRET_FILE: &'static str = "secret";

    let secret_file = path::Path::new(SECRET_FILE);

    if let Ok(token) = env::var(ENV_VAR) {
        return Ok(token);
    } else if let Ok(token) = fs::read_to_string(secret_file) {
        return Ok(String::from(token.trim()));
    }
    let current_directory = env::current_dir().unwrap();
    let secret_file_path = format!(
        "{}{}{}",
        current_directory.display(),
        path::MAIN_SEPARATOR,
        SECRET_FILE
    );
    let error_message = format!(
        "Could not find server token in environment \
        variable '{}' or file '{}'",
        ENV_VAR, secret_file_path
    );
    Err(error_message)
}
