use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
	utils::MessageBuilder,
};

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
			let mut queue_message = MessageBuilder::new();
			let handler = handler_lock.lock().await;
			handler
				.queue()
				.current_queue()
				.iter()
				.map(|t| t.metadata())
				.enumerate()
				.for_each(|(index, metadata)| {
					if index == 0 {
						queue_message.push_mono("Now Playing");
					} else {
						queue_message.push_mono(index);
					}
					queue_message.push(" | ");
					queue_message.push_safe(
						metadata
							.title
							.as_deref()
							.unwrap_or("Title information not present"),
					);
					queue_message.push("  ");
					queue_message.push_mono_line(
						metadata
							.duration
							.map(format_duration)
							.as_deref()
							.unwrap_or("No info"),
					);
				});

			msg.channel_id.say(&ctx.http, queue_message).await?;
		}
		None => {
			msg.channel_id.say(&ctx.http, "Queue is empty.").await?;
		}
	}

	Ok(())
}
