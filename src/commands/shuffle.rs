use std::time::Duration;

use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use serenity::{
	framework::standard::CommandResult,
	model::channel::Message,
	{client::Context, framework::standard::macros::command},
};
use songbird::tracks::TrackResult;

#[command]
#[description = "Shuffles the current queue"]
#[only_in(guilds)]
async fn shuffle(ctx: &Context, msg: &Message) -> CommandResult {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	let message = match manager.get(msg.guild(&ctx.cache).await.unwrap().id) {
		Some(handler_lock) => {
			let handler = handler_lock.lock().await;

			if handler.queue().len() > 1 {
				handler.queue().pause()?;
				handler.queue().modify_queue::<_, TrackResult<_>>(|queue| {
					queue[0].seek_time(Duration::from_secs(0))?;
					let mut rng = SmallRng::from_entropy();
					queue.make_contiguous().shuffle(&mut rng);
					Ok(())
				})?;
				handler.queue().resume()?;
				"Queue shuffled!"
			} else if handler.queue().len() == 1 {
				"Cannot shuffle queue with only one song"
			} else {
				"Cannot shuffle empty queue"
			}
		}
		None => "Not playing in voice channel",
	};

	msg.channel_id.say(&ctx.http, message).await?;

	Ok(())
}
