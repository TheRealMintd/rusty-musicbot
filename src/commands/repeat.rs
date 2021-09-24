use std::fmt::Display;

use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	model::channel::Message,
	utils::MessageBuilder,
};

use crate::utils::ObtainTitle;

#[command]
#[only_in(guilds)]
#[min_args(1)]
#[max_args(2)]
#[aliases("loop")]
#[usage("[track-position] number-of-loops")]
#[example("1")]
#[example("2 1")]
/// Repeat the selected track, or the current track, by the specified number of times
async fn repeat(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	let message = match manager.get(msg.guild(&ctx.cache).await.unwrap().id) {
		Some(handler_lock) => {
			let handler = handler_lock.lock().await;
			let queue = handler.queue();

			if args.len() == 1 {
				match args.trimmed().parse::<usize>() {
					Ok(repeat_for) => {
						let current_queue = queue.current_queue();
						match current_queue.last() {
							Some(track) => {
								track.loop_for(repeat_for)?;
								get_success_message(track.get_title(), repeat_for)
							}
							None => "No song to repeat.".to_string(),
						}
					}
					Err(_) => "Parameter must be an integer".to_string(),
				}
			} else {
				match args.trimmed().parse::<usize>().and_then(|track_number| {
					args.advance()
						.parse::<usize>()
						.map(|repeat_for| (track_number, repeat_for))
				}) {
					Ok((track_number, repeat_for)) => match queue.current_queue().get(track_number)
					{
						Some(track) => {
							track.loop_for(repeat_for)?;
							get_success_message(track.get_title(), repeat_for)
						}
						None => "There is no track at that position".to_string(),
					},
					Err(_) => "Parameters must be integers".to_string(),
				}
			}
		}
		None => "Not playing in voice channel".to_string(),
	};

	msg.channel_id
		.send_message(&ctx.http, |m| m.embed(|e| e.description(message)))
		.await?;
	Ok(())
}

fn get_success_message<T: Display>(title: T, repeat_for: usize) -> String {
	MessageBuilder::new()
		.push("Repeating ")
		.push_mono_safe(title)
		.push(" ")
		.push(repeat_for)
		.push(" times.")
		.build()
}
