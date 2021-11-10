use std::time::Duration;

use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
};
use tokio::time::sleep;
use tracing::warn;

use crate::utils::get_user_server_channel;

#[command]
#[only_in(guilds)]
#[num_args(0)]
/// Stops and disconnects the bot from the voice channel
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
	if let Some((guild, _)) = get_user_server_channel(ctx, msg).await {
		let manager = songbird::get(ctx)
			.await
			.expect("Songbird Voice Client placed in at initialisation.")
			.clone();

		while let Err(e) = manager.remove(guild).await {
			warn!(
				"Could not leave voice channel: {}, trying again in 5 seconds",
				e
			);
			sleep(Duration::from_secs(5)).await;
		}
	}
	Ok(())
}
