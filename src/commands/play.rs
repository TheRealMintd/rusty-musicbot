use futures_util::StreamExt;
use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
	utils::MessageBuilder,
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

	let message = args.message().trim();
	let song_stream = PlayParameter::MaybeUrl(message.to_string()).get_tracks();
	tokio::pin!(song_stream);
	let mut added_songs = 0;
	let mut handler_lock = None;
	let mut message_content = MessageBuilder::new();
	let mut error_occured = false;

	while let Some(song) = song_stream.next().await {
		match song {
			Ok((track, track_handle)) => {
				if added_songs == 0 {
					handler_lock = Some(
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
						},
					);
				}

				let position = {
					let mut handler =
						handler_lock.as_ref().unwrap().lock().await;
					handler.enqueue(track);
					added_songs += 1;
					handler.queue().len() - 1
				};
				info!(
					"Track <{}> queued in guild {}",
					track_handle.get_title(),
					guild_id
				);
				if added_songs == 1 {
					message_content.push(build_description(
						track_handle.get_title(),
						track_handle.metadata(),
						position,
					));
					result_message
						.edit(&ctx.http, |m| {
							m.embed(|m| m.description(&message_content))
						})
						.await?;
				}
			}
			Err(e) => {
				error!("Error occurred during video download: {}", e);
				error_occured = true;
			}
		}
	}

	if added_songs == 0 {
		result_message
			.edit(&ctx.http, |m| m.content("Error downloading songs"))
			.await?;
	} else {
		if added_songs > 1 {
			message_content.push(format!(
				"\nAnother {} song(s) added from playlist",
				added_songs - 1
			));
		}

		result_message
			.edit(&ctx.http, |m| {
				m.content(if error_occured {
					"Some songs may have been skipped due to errors"
				} else {
					""
				});
				m.embed(|m| m.description(message_content))
			})
			.await?;
	}
	Ok(())
}
