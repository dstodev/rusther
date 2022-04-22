#![crate_name = "rusther"]

use std::env;
use std::fs;

use log::LevelFilter;
use serenity::prelude::*;
use simple_logger::SimpleLogger;
use tokio::runtime::Builder;

use rusther::Arbiter;

mod rusther;
mod commands;

fn main() -> Result<(), String> {
	SimpleLogger::new()
		.with_colors(true)
		.with_local_timestamps()
		.with_level(LevelFilter::Off)
		.env()  // Must appear after .with_level() to take effect; enables RUST_LOG environment var
		.with_module_level("rusther", LevelFilter::Debug)  // But this line takes ultimate precedence for module-level logging
		.init()
		.unwrap();

	log::info!("Logger initialized");
	log::debug!("  With debug messages");
	log::trace!("  With trace messages");

	let mut arbiter = Arbiter::new();

	arbiter.register_event_handler("ping", Box::new(commands::Ping::new()))?;
	arbiter.register_event_handler("announce", Box::new(commands::Announce))?;
	arbiter.register_event_handler("connect_four", Box::new(commands::ConnectFourDiscord::new()))?;

	let token = get_token().unwrap();

	let runtime = Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap();

	runtime.block_on(async {
		let mut client = Client::builder(token)
			.event_handler(arbiter)
			.await
			.expect("Could not create client!");

		if let Err(reason) = client.start().await {
			log::debug!("Client failed to start because {:?}", reason);
		}
	});

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
