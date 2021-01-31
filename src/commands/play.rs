use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
	utils::{EmbedMessageBuilding, MessageBuilder},
};

use tracing::{error, info};

use url::Url;

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

	// parse the string to see if it is a valid URL, if it is, try to download from it, otherwise search YouTube with the string
	let message = args.message().trim();
	let valid_url = Url::parse(message).is_err();
	let (track_handle, queue_length) = match get_track(message, valid_url).await {
		Ok((track, track_handle)) => {
			let handler_lock = match join_channel(ctx, guild_id, channel_id).await {
				Ok(handler_lock) => handler_lock,
				Err(e) => {
					msg.channel_id
						.say(&ctx.http, "Error joining the channel.")
						.await?;
					error!("Cannot join channel: {:?}", e);
					return Ok(());
				}
			};
			let mut handler = handler_lock.lock().await;
			handler.enqueue(track);
			(track_handle, handler.queue().len())
		}
		Err(e) => {
			msg.channel_id
				.say(&ctx.http, "Error occurred during video download.")
				.await?;
			error!("Failed to download video file: {:?}", e);
			return Ok(());
		}
	};

	let title = track_handle.get_title();
	info!("Track <{}> queued in guild {}", title, guild_id);
	msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|m| {
				let mut embed = MessageBuilder::new();
				embed.push("Added ");

				if let Some(ref url) = track_handle.metadata().source_url {
					embed.push_named_link_safe(title, url);
				} else {
					embed.push_quote_safe(title);
				}

				if let Some(duration) = track_handle.metadata().duration {
					embed.push(" ");
					embed.push_mono(format_duration(duration));
				}

				embed.push(" to queue at position ");
				embed.push(queue_length);
				embed.push(".");

				m.description(embed)
			})
		})
		.await?;

	Ok(())
}
