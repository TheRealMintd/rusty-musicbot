mod commands;
mod events;
mod utils;

use std::{collections::HashSet, env};

use once_cell::sync::Lazy;
use serenity::{
	async_trait,
	framework::{standard::macros::group, StandardFramework},
	http::Http,
	model::gateway::Ready,
	prelude::*,
};
use songbird::serenity::SerenityInit;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
	about::*, help::*, pause::*, ping::*, play::*, playnext::*, queue::*,
	repeat::*, resume::*, search::*, shuffle::*, skip::*, stop::*, version::*,
};

static QUEUE_CHUNK_SIZE: Lazy<usize> = Lazy::new(|| {
	env::var("RUSTY_QUEUE_CHUNK_SIZE")
		.ok()
		.and_then(|size| size.parse().ok())
		.unwrap_or(20)
});

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _: Context, ready: Ready) {
		info!("Connected as : {}", ready.user.name);
	}
}

#[group]
#[commands(
	about, pause, ping, play, playnext, queue, repeat, resume, search, shuffle,
	skip, stop, version
)]
struct General;

#[tokio::main]
async fn main() {
	let _ = dotenv::dotenv();

	let subscriber = FmtSubscriber::builder()
		.with_env_filter(EnvFilter::from_default_env())
		.finish();

	tracing::subscriber::set_global_default(subscriber)
		.expect("Failed to start logger.");

	let token =
		env::var("DISCORD_TOKEN").expect("Token environment variable not set");
	let http = Http::new_with_token(&token);

	let (owners, bot_id) = match http.get_current_application_info().await {
		Ok(info) => {
			let mut owners = HashSet::new();
			owners.insert(info.owner.id);

			(owners, info.id)
		}
		Err(e) => panic!("Cannot access application info: {:?}", e),
	};

	let framework = StandardFramework::new()
		.configure(|c| {
			if let Ok(prefix) = env::var("MUSICBOT_PREFIX") {
				c.prefix(&prefix);
			}

			c.owners(owners)
				.on_mention(Some(bot_id))
				.case_insensitivity(true)
		})
		.group(&GENERAL_GROUP)
		.help(&HELP);

	let mut client = Client::builder(&token)
		.framework(framework)
		.event_handler(Handler)
		.register_songbird()
		.await
		.expect("Error creating client");

	let shard_manager = client.shard_manager.clone();
	tokio::spawn(async move {
		tokio::signal::ctrl_c()
			.await
			.expect("Could not register ctrl+c handler");
		shard_manager.lock().await.shutdown_all().await;
	});

	if let Err(e) = client.start().await {
		error!("Client error: {:?}", e);
	}
}
