use std::{borrow::Cow, fmt::Display, future::Future, time::Duration};

use async_stream::{stream, try_stream};
use futures_core::Stream;
use futures_util::stream::{FuturesOrdered, StreamExt};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Deserializer;
use serenity::{
	model::{
		channel::Message,
		id::{ChannelId, GuildId},
	},
	prelude::*,
	utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::{
	input::{error::Result as SongbirdResult, Metadata, Restartable},
	tracks::{create_player, Track, TrackHandle},
	Call,
};
use tokio::{process::Command, sync::MutexGuard};
use tracing::{error, info, warn};
use url::Url;

use crate::QUEUE_CHUNK_SIZE;

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

pub(crate) enum PlayParameter {
	MaybeUrl(String),
	Url(String),
}

impl PlayParameter {
	pub(crate) fn get_tracks(
		self,
	) -> impl Stream<Item = SongbirdResult<(Track, TrackHandle)>> {
		stream! {
			match self {
				Self::Url(url) => {
					yield Restartable::ytdl(url, true)
						.await
						.map(|song| create_player(song.into()));
				}
				Self::MaybeUrl(potential_url) => match Url::parse(&potential_url) {
					Ok(url) => {
						for await result in Self::handle_url(url) {
							yield result;
						}
					}
					Err(_) => {
						yield Restartable::ytdl_search(potential_url, true)
							.await
							.map(|song| create_player(song.into()));
					}
				},
			}
		}
	}

	fn handle_url(
		url: Url,
	) -> impl Stream<Item = SongbirdResult<(Track, TrackHandle)>> {
		const KNOWN_PLAYLIST_HOSTS: [&str; 3] =
			["youtube.com", "music.youtube.com", "www.youtube.com"];

		let is_playlist = url
			.host_str()
			.map(|host| KNOWN_PLAYLIST_HOSTS.contains(&host))
			.unwrap_or(false)
			&& url.path() == "/playlist"
			&& url.query_pairs().any(|(key, _)| key == "list");

		try_stream! {
			if is_playlist {
				let ytdl = Command::new("youtube-dl")
					.args(&["-j", "--flat-playlist", "--ignore-config"])
					.arg(url.as_str())
					.output()
					.await?;

				let mut song_iter = Deserializer::from_slice(&ytdl.stdout)
					.into_iter::<serde_json::Value>()
					.filter_map(|video| video.ok())
					.map(|video| {
						let url = video
							.get("url")
							.expect("youtube-dl JSON has no 'url' field")
							.as_str()
							.expect("youtube-dl JSON has wrong 'url' field type")
							.to_string();
						Restartable::ytdl(url, true)
					});

				match song_iter.next() {
					Some(song) => {
						yield create_player(song.await?.into());

						let songs = song_iter
							.chunks(*QUEUE_CHUNK_SIZE)
							.into_iter()
							.map(|chunk| chunk.collect::<FuturesOrdered<_>>())
							.collect::<Vec<_>>();

						for mut song_chunk in songs {
							while let Some(song) = song_chunk.next().await {
								yield create_player(song?.into());
							}
						}
					}
					None => {}
				}
			} else {
				yield create_player(Restartable::ytdl(url, true).await?.into());
			}
		}
	}
}

pub(crate) async fn queue_songs(
	handler: &mut MutexGuard<'_, Call>,
	song_stream: impl Stream<Item = SongbirdResult<(Track, TrackHandle)>>,
) -> Result<String, &'static str> {
	let ((mut message, added_songs, error), elapsed) =
		time_section(|| async move {
			tokio::pin!(song_stream);

			let mut error = false;
			let mut song_count = 0;
			let mut message = MessageBuilder::new();

			if let Some(song) = song_stream.next().await {
				match song {
					Ok((track, track_handle)) => {
						handler.enqueue(track);
						song_count += 1;
						info!("Track <{}> queued", track_handle.get_title());
						message.push(build_description(
							track_handle.get_title(),
							track_handle.metadata(),
						));
					}
					Err(e) => {
						error!("Error occurred during video download: {}", e);
						error = true;
					}
				}
			}

			while let Some(song) = song_stream.next().await {
				match song {
					Ok((track, track_handle)) => {
						handler.enqueue(track);
						song_count += 1;
						info!("Track <{}> queued", track_handle.get_title());
					}
					Err(e) => {
						error!("Error occurred during video download: {}", e);
						error = true;
					}
				}
			}

			(message, song_count, error)
		})
		.await;

	if added_songs == 0 {
		Err("Error downloading songs")
	} else {
		message.push(format!(
			"\n\nAdded {} song(s) in {}",
			added_songs,
			format_duration(elapsed)
		));

		if error {
			message.push("\nSome songs may have been skipped due to errors");
		}

		Ok(message.build())
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

pub(crate) fn escape_markdown(text: &str) -> Cow<str> {
	static REGEX: Lazy<Regex> =
		Lazy::new(|| Regex::new(r"([*_`~\\\[\]])").unwrap());
	REGEX.replace_all(text, r"\$1")
}

pub(crate) fn build_description<T>(
	title: T,
	metadata: &Metadata,
) -> MessageBuilder
where
	T: AsRef<str> + Display,
{
	let mut embed = MessageBuilder::new();
	embed.push("Name: ");

	if let Some(ref url) = metadata.source_url {
		embed.push_named_link(escape_markdown(title.as_ref()), url);
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
}

pub(crate) async fn time_section<F, T, Fut>(func: F) -> (T, Duration)
where
	F: FnOnce() -> Fut,
	Fut: Future<Output = T>,
{
	let timer = std::time::Instant::now();
	let result = func().await;

	(result, timer.elapsed())
}

pub(crate) async fn leave_if_empty(
	ctx: &Context,
	handler: MutexGuard<'_, Call>,
	guild: GuildId,
) {
	if !handler.queue().is_empty() {
		return;
	}

	drop(handler);
	let manager = songbird::get(ctx)
		.await
		.expect("Cannot obtain Songbird voice client")
		.clone();

	while let Err(e) = manager.remove(guild).await {
		warn!(
			"Could not leave voice channel: {}, trying again in 5 seconds",
			e
		);
		tokio::time::sleep(Duration::from_secs(5)).await;
	}
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
