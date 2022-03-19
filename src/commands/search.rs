use std::{borrow::Cow, convert::TryFrom, time::Duration};

use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Deserializer;
use serenity::{
	client::Context,
	framework::standard::{macros::command, Args, CommandResult},
	futures::future,
	model::channel::{Message, ReactionType},
	utils::MessageBuilder,
};
use tokio::process::Command;
use tracing::error;

use crate::{join_channel, utils::*};

static NUMBER_REACTS: Lazy<[ReactionType; 4]> = Lazy::new(|| {
	[
		ReactionType::try_from("1️⃣").unwrap(),
		ReactionType::try_from("2️⃣").unwrap(),
		ReactionType::try_from("3️⃣").unwrap(),
		ReactionType::try_from("4️⃣").unwrap(),
	]
});

#[derive(Debug, Deserialize)]
struct SearchResult<'a> {
	#[serde(borrow)]
	title: Cow<'a, str>,
	#[serde(rename = "webpage_url")]
	#[serde(borrow)]
	url: Cow<'a, str>,
}

#[command]
#[only_in(guilds)]
#[min_args(1)]
/// Search for a video on YouTube
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	let handler_lock = join_channel!(ctx, msg);
	let mut result_message = msg
		.channel_id
		.say(&ctx.http, "Please wait, searching...")
		.await?;

	let output = Command::new("youtube-dl")
		.arg("-R")
		.arg("infinite")
		.arg("--ignore-config")
		.arg("--dump-json")
		.arg(format!("ytsearch4:{}", args.message().trim()))
		.output()
		.await;

	let objects = match output {
		Ok(objects) => objects,
		Err(e) => {
			msg.channel_id
				.say(&ctx.http, "Error retrieving search results.")
				.await?;
			error!("Error retrieving search results: {:?}", e);
			return Ok(());
		}
	};

	// build selection message
	// youtube-dl's --dump-json command outputs each video as an object on one line, so the into_iter method is used to process each one
	let results: Vec<SearchResult> = Deserializer::from_slice(&objects.stdout)
		.into_iter()
		.filter_map(|sr| sr.ok())
		.collect();

	// some searches don't have any results, send a different message
	if results.is_empty() {
		msg.channel_id
			.say(
				&ctx.http,
				MessageBuilder::new()
					.push("No results found for ")
					.push_quote_safe(args.message().trim()),
			)
			.await?;
		return Ok(());
	}

	result_message
		.edit(&ctx.http, |m| {
			m.content("Here are the search results:").embed(|e| {
				let mut embed_message = MessageBuilder::new();

				results.iter().enumerate().for_each(|(index, sr)| {
					embed_message
						.push_mono(index + 1)
						.push(" | ")
						.push_line_safe(&sr.title);
				});

				e.description(embed_message)
			})
		})
		.await?;

	// add reactions to the message
	let results_count = results.len();
	for emoji in NUMBER_REACTS.iter().take(results_count).cloned() {
		result_message.react(&ctx.http, emoji).await?;
	}
	let reacts = NUMBER_REACTS
		.iter()
		.take(results_count)
		.cloned()
		.map(|emoji| result_message.react(&ctx.http, emoji));
	future::join_all(reacts).await;

	// wait for the user to make a selection using a reaction
	let reactions = result_message
		.await_reaction(&ctx)
		.timeout(Duration::from_secs(60))
		.author_id(msg.author.id)
		.filter(move |reaction| {
			NUMBER_REACTS[..results_count].contains(&reaction.as_ref().emoji)
		})
		.await;

	result_message.delete_reactions(&ctx.http).await?;
	result_message
		.edit(&ctx.http, |m| m.embed(|e| e.description("Please wait...")))
		.await?;

	let url = match reactions {
		Some(reaction) => {
			&results[NUMBER_REACTS
				.iter()
				.position(|number| number == &reaction.as_inner_ref().emoji)
				.expect("Reacted to another reaction")]
			.url
		}
		None => {
			result_message
				.edit(&ctx.http, |m| {
					m.content("One minute has passed with no selection.");
					m.suppress_embeds(true)
				})
				.await?;
			return Ok(());
		}
	};

	let song_stream = PlayParameter::Url(url.to_string()).get_tracks();
	let mut handler = handler_lock.lock().await;
	match queue_songs(&mut handler, song_stream).await {
		Ok(message) => {
			result_message
				.edit(&ctx.http, |m| {
					m.content("");
					m.embed(|m| m.description(message))
				})
				.await?;
		}
		Err(message) => {
			result_message
				.edit(&ctx.http, |m| m.content(message))
				.await?;
			leave_if_empty(
				ctx,
				handler,
				msg.guild(&ctx.cache).await.unwrap().id,
			)
			.await;
		}
	}

	Ok(())
}
