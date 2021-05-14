use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
};

use tracing::error;

// Dicord automatically changes the tabs to four spaces, so extra spaces are added to fix alignment
const VERSION_TEXT: &str = concat!(
	"```",
	"Rusty Musicbot - ",
	env!("VERGEN_GIT_SEMVER"),
	"\n\nBuild Timestamp:\t ",
	env!("VERGEN_BUILD_TIMESTAMP"),
	"\nFrom Commit:\t\t ",
	env!("VERGEN_GIT_SHA_SHORT"),
	"\nCommit Timestamp:\t",
	env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
	"\n\nRust Version:\t\t",
	env!("VERGEN_RUSTC_SEMVER"),
	"```",
);

#[command]
/// Get the build details of the bot
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
	if let Err(e) = msg.channel_id.say(&ctx.http, VERSION_TEXT).await {
		error!("Error printing version: {}", e);
	};

	Ok(())
}
