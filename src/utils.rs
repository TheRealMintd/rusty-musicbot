use std::{sync::Arc, time::Duration};

use serenity::{
	model::{
		channel::Message,
		id::{ChannelId, GuildId},
	},
	prelude::*,
};

use songbird::{
	error::JoinError,
	input::{error::Result as SongbirdResult, Restartable},
	tracks::{create_player, Track, TrackHandle},
	Call, Event,
};

use crate::events::TrackEnd;

pub(crate) trait ObtainTitle {
	fn get_title(&self) -> &str;
}

impl ObtainTitle for TrackHandle {
	fn get_title(&self) -> &str {
		self.metadata()
			.title
			.as_deref()
			.unwrap_or("Name not present")
	}
}

pub(crate) async fn get_user_server_channel(
	ctx: &Context,
	msg: &Message,
) -> Option<(GuildId, ChannelId)> {
	let guild = msg.guild(&ctx.cache).await?;

	Some((
		guild.id,
		guild
			.voice_states
			.get(&msg.author.id)
			.and_then(|voice_state| voice_state.channel_id)?,
	))
}

pub(crate) async fn join_channel(
	ctx: &Context,
	guild_id: GuildId,
	channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, JoinError> {
	let manager = songbird::get(ctx)
		.await
		.expect("Songbird Voice Client placed in at initialisation.")
		.clone();

	match manager.get(guild_id) {
		Some(handler_lock) => Ok(handler_lock),
		None => {
			let (handler_lock, success) = manager.join(guild_id, channel_id).await;

			match success {
				Ok(_) => {
					let mut handler = handler_lock.lock().await;
					handler.add_global_event(
						Event::Track(songbird::TrackEvent::End),
						TrackEnd {
							guild_id,
							manager: manager.clone(),
						},
					)
				}
				Err(e) => {
					return Err(e);
				}
			}

			Ok(handler_lock)
		}
	}
}

pub(crate) async fn get_track(
	url_or_search: &str,
	search: bool,
) -> SongbirdResult<(Track, TrackHandle)> {
	Ok(create_player(
		if search {
			Restartable::ytdl_search(url_or_search, true).await?
		} else {
			Restartable::ytdl(url_or_search.to_string(), true).await?
		}
		.into(),
	))
}

pub(crate) fn format_duration(duration: Duration) -> String {
	let hours = duration.as_secs() / 60 / 60;
	let minutes = duration.as_secs() / 60 % 60;
	let seconds = duration.as_secs() % 60;

	if hours != 0 {
		format!("{}:{}:{}", hours, minutes, seconds)
	} else {
		format!("{}:{}", minutes, seconds)
	}
}
