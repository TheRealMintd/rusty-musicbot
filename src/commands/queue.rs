use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
	utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::tracks::TrackHandle;

use crate::utils::*;

#[command]
#[only_in(guilds)]
#[num_args(0)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
	let guild = msg.guild(&ctx.cache).await.unwrap();
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	match manager.get(guild.id) {
		Some(handler_lock) => {
			let handler = handler_lock.lock().await;

			msg.channel_id
				.send_message(&ctx.http, |m| {
					m.embed(|e| {
						e.description(build_queue_message(
							&handler.queue().current_queue(),
						))
					})
				})
				.await?;
		}
		None => {
			msg.channel_id.say(&ctx.http, "Queue is empty.").await?;
		}
	}

	Ok(())
}

pub(crate) fn build_queue_message(queue: &[TrackHandle]) -> MessageBuilder {
	let mut queue_message = MessageBuilder::new();
	queue.iter().enumerate().for_each(|(index, metadata)| {
		if index == 0 {
			queue_message.push_mono("Now Playing");
		} else {
			queue_message.push_mono(index);
		}
		queue_message.push(" | ");

		match metadata.metadata().source_url {
			Some(ref url) => queue_message
				.push_named_link(escape_markdown(metadata.get_title()), url),
			None => queue_message.push_mono_safe(metadata.get_title()),
		};

		queue_message.push("  ");
		queue_message.push_mono_line(
			metadata
				.metadata()
				.duration
				.map(format_duration)
				.as_deref()
				.unwrap_or("No info"),
		);
	});

	queue_message
}
