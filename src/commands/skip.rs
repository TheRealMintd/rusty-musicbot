use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
};

#[command]
#[only_in(guilds)]
#[max_args(1)]
#[aliases("remove")]
/// Skips the currently playing song.
async fn skip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	let message =
		match manager.get(msg.guild(&ctx.cache).await.unwrap().id) {
			Some(handler_lock) => {
				let handler = handler_lock.lock().await;
				let queue = handler.queue();

				if args.is_empty() {
					queue.skip()?;
					"Skipped song".to_string()
				} else {
					args.parse::<usize>()
						.map(|index| {
							queue
								.dequeue(index)
								.map(|track| {
									track.stop().unwrap();
									format!("Skipped track at position {} in queue.", index)
								})
								.unwrap_or(format!(
									"No track at position {}.",
									index
								))
						})
						.unwrap_or_else(|_| {
							"Parameter must be a positive number.".to_string()
						})
				}
			}
			None => "Not playing in voice channel".to_string(),
		};

	msg.channel_id.say(&ctx.http, message).await?;

	Ok(())
}
