use futures_util::StreamExt;
use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
};

use super::helpers::join_channel;
use crate::utils::{leave_if_empty, queue_songs, PlayParameter};

#[command]
#[only_in(guilds)]
#[min_args(1)]
/// Downloads and plays the provided link, or searches for the video on YouTube.
/// Plays after the current song.
async fn playnext(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let handler_lock = join_channel!(ctx, msg);
	let mut result_message = msg
		.channel_id
		.say(&ctx.http, "Please wait, searching...")
		.await?;

	let song_stream = PlayParameter::MaybeUrl(args.message().trim().to_owned())
		.get_tracks()
		.take(1);
	let mut handler = handler_lock.lock().await;
	match queue_songs(&mut handler, song_stream).await {
		Ok(message) => {
			handler.queue().modify_queue(|queue| {
				if queue.len() > 1 {
					if let Some(track) = queue.pop_back() {
						queue.push_front(track);
						queue.swap(0, 1);
					}
				}
			});
			result_message
				.edit(&ctx.http, |m| {
					m.content("");
					m.embed(|m| m.description(message))
				})
				.await?;
		}
		Err(message) => {
			result_message
				.edit(&ctx.http, |m| m.content(message))
				.await?;
			leave_if_empty(
				ctx,
				handler,
				msg.guild(&ctx.cache).await.unwrap().id,
			)
			.await;
		}
	}

	Ok(())
}
