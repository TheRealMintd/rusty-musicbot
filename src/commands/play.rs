use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
};
use tracing::{error, info};

use crate::utils::*;

#[command]
#[only_in(guilds)]
#[min_args(1)]
/// Downloads and plays the provided link, or searches for the video on YouTube
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	// user must be connected to a voice channel to issue playback commands
	let (guild_id, channel_id) = match get_user_server_channel(ctx, msg).await {
		Some(channel) => channel,
		None => {
			msg.reply(
				&ctx.http,
				"You must be in a voice channel to use this command.",
			)
			.await?;
			return Ok(());
		}
	};

	// present the user with a loading message
	let mut result_message = msg
		.channel_id
		.say(&ctx.http, "Please wait, searching...")
		.await?;

	// parse the string to see if it is a valid URL, if it is, try to download
	// from it, otherwise search YouTube with the string
	let message = args.message().trim();
	match PlayParameter::MaybeUrl(message).get_tracks().await {
		Ok(mut tracks) => {
			let handler_lock =
				match join_channel(ctx, guild_id, channel_id).await {
					Ok(handler_lock) => handler_lock,
					Err(e) => {
						result_message
							.edit(&ctx.http, |m| {
								m.content("Error joining the channel.")
							})
							.await?;
						error!("Cannot join channel: {:?}", e);
						return Ok(());
					}
				};
			let num_tracks = tracks.len();

			if num_tracks == 1 {
				let (track, track_handle) = tracks.remove(0);
				// let title = track_handle.get_title();
				let position = {
					let mut handler = handler_lock.lock().await;
					handler.enqueue(track);
					handler.queue().len() - 1
				};

				info!(
					"Track <{}> queued in guild {}",
					track_handle.get_title(),
					guild_id
				);
				result_message
					.edit(&ctx.http, |m| {
						m.content("");
						m.embed(|m| {
							m.description(build_description(
								track_handle.get_title(),
								track_handle.metadata(),
								position,
							))
						})
					})
					.await?;
			} else {
				{
					let mut handler = handler_lock.lock().await;
					tracks
						.into_iter()
						.for_each(|(track, _)| handler.enqueue(track));
				}
				info!("Playlist <{}> queued in guild {}", message, guild_id);
				result_message
					.edit(&ctx.http, |m| {
						m.content("");
						m.embed(|m| {
							m.description(format!(
								"Queued {} tracks",
								num_tracks
							))
						})
					})
					.await?;
			}
		}
		Err(e) => {
			result_message
				.edit(&ctx.http, |m| {
					m.content("Error occurred during video download.")
				})
				.await?;
			error!("Failed to download video file: {:?}", e);
			return Ok(());
		}
	};

	Ok(())
}
