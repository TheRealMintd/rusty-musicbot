use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
};

#[command]
#[only_in(guilds)]
#[num_args(0)]
/// Pauses the currently playing song
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	match manager.get(msg.guild(&ctx.cache).await.unwrap().id) {
		Some(handler_lock) => {
			let handler = handler_lock.lock().await;

			match handler.queue().current().map(|track| track.pause()) {
				Some(_) => {
					msg.channel_id.say(&ctx.http, "Paused!").await?;
				}
				None => {
					msg.channel_id.say(&ctx.http, "No tracks in queue.").await?;
				}
			}
		}
		None => {
			msg.channel_id
				.say(
					&ctx.http,
					"You must be in a voice channel to use this command.",
				)
				.await?;
		}
	}

	Ok(())
}
