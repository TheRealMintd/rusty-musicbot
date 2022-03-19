pub(crate) use crate::join_channel;

#[macro_export]
macro_rules! join_channel {
	($ctx:ident, $msg:ident) => {{
		use crate::utils::get_user_server_channel;
		use tracing::error;

		let (guild_id, channel_id) =
			match get_user_server_channel($ctx, $msg).await {
				Some(channel) => channel,
				None => {
					$msg.reply(
						&$ctx.http,
						"You must be in a voice channel to use this command.",
					)
					.await?;
					return Ok(());
				}
			};

		let manager = songbird::get($ctx)
			.await
			.expect("Unable to obtain Songbird client")
			.clone();

		match manager.get(guild_id) {
			Some(handler_lock) => handler_lock,
			None => {
				let (handler_lock, success) =
					manager.join(guild_id, channel_id).await;

				match success {
					Ok(_) => {
						use crate::events::TrackEnd;
						use songbird::Event;
						let mut handler = handler_lock.lock().await;
						handler.deafen(true).await?;
						handler.add_global_event(
							Event::Track(songbird::TrackEvent::End),
							TrackEnd {
								guild_id,
								manager: manager.clone(),
							},
						);
					}
					Err(e) => {
						$msg.reply(&$ctx.http, "Error joining the channel.")
							.await?;
						error!("Cannot join channel: {:?}", e);
						return Ok(());
					}
				}

				handler_lock
			}
		}
	}};
}
