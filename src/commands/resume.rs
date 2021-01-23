use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
};

#[command]
#[only_in(guilds)]
#[num_args(0)]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
	let guild = msg.guild(&ctx.cache).await.unwrap();
	if guild.voice_states.get(&msg.author.id).is_none() {
		msg.reply(ctx, "User not in voice channel").await?;
		return Ok(());
	}

	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	match manager.get(guild.id) {
		Some(handler_lock) => {
			let handler = handler_lock.lock().await;
			match handler.queue().current() {
				Some(track) => track.play()?,
				None => {
					msg.channel_id.say(&ctx.http, "Queue is empty.").await?;
				}
			}
		}
		None => {
			msg.channel_id
				.say(&ctx.http, "Bot is not in a voice channel.")
				.await?;
		}
	}

	Ok(())
}
