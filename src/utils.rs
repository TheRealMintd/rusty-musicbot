use std::{fmt::Display, sync::Arc, time::Duration};

use serenity::{
	model::{
		channel::Message,
		id::{ChannelId, GuildId},
	},
	prelude::*,
	utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::{
	error::JoinError,
	input::{error::Result as SongbirdResult, Metadata, Restartable},
	tracks::{create_player, Track, TrackHandle},
	Call, Event,
};
use url::Url;

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
			let (handler_lock, success) =
				manager.join(guild_id, channel_id).await;

			match success {
				Ok(_) => {
					let mut handler = handler_lock.lock().await;
					handler.deafen(true).await?;
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

pub(crate) enum PlayParameter<'a> {
	MaybeUrl(&'a str),
	Url(&'a str),
}

impl<'a> PlayParameter<'a> {
	pub async fn get_tracks(self) -> SongbirdResult<Vec<(Track, TrackHandle)>> {
		Ok(match self {
			Self::MaybeUrl(search_term) if Url::parse(search_term).is_err() => {
				vec![create_player(
					Restartable::ytdl_search(search_term, true).await?.into(),
				)]
			}
			Self::MaybeUrl(url) | Self::Url(url) => {
				vec![create_player(
					Restartable::ytdl(url.to_string(), true).await?.into(),
				)]
			}
		})
	}
}

pub(crate) fn format_duration(duration: Duration) -> String {
	let hours = duration.as_secs() / 60 / 60;
	let minutes = duration.as_secs() / 60 % 60;
	let seconds = duration.as_secs() % 60;

	if hours != 0 {
		format!("{}:{:02}:{:02}", hours, minutes, seconds)
	} else {
		format!("{}:{:02}", minutes, seconds)
	}
}

pub(crate) fn build_description<T>(
	title: T,
	metadata: &Metadata,
	position: usize,
) -> String
where
	T: AsRef<str> + Display,
{
	let mut embed = MessageBuilder::new();
	embed.push("Name: ");

	if let Some(ref url) = metadata.source_url {
		embed.push_named_link_safe(title, url);
	} else {
		embed.push_quote_safe(title);
	}

	if let Some(duration) = metadata.duration {
		embed
			.push("\nDuration: ")
			.push_mono_line(format_duration(duration));
	}

	if let Some(ref uploader) = metadata.artist {
		embed.push("Artist/Uploader: ").push_line_safe(uploader);
	}

	embed
		.push("\nAdded to queue at position ")
		.push(position)
		.push(".")
		.build()
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::format_duration;

	#[test]
	fn test_format_duration() {
		assert_eq!(format_duration(Duration::from_secs(8)), "0:08");
		assert_eq!(format_duration(Duration::from_secs(60)), "1:00");
		assert_eq!(format_duration(Duration::from_secs(3600)), "1:00:00");
	}
}
