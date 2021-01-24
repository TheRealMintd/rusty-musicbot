use serenity::{
	client::Context,
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
	utils::{EmbedMessageBuilding, MessageBuilder},
};

#[command]
#[num_args(0)]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
	msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.description(
					MessageBuilder::new()
						.push("This is ")
						.push_mono("rusty-musicbot")
						.push_line(", yet another music bot for Discord.\n")
						.push_line("Built using these wonderful libraries:")
						.push_named_link("Serenity", "https://github.com/serenity-rs/serenity/")
						.push_line(" - A wonderful Rust library for Discord.")
						.push_named_link("Songbird", "https://github.com/serenity-rs/songbird")
						.push_line(" - Library for the Discord Voice API.")
						.push_named_link("youtube-dl", "https://youtube-dl.org/")
						.push_line(" - Powerful video downloader.")
						.push_named_link("FFmpeg", "https://ffmpeg.org/")
						.push_line(" - Powerful utility for processing multimedia.\n")
						.push_line("Written by: Wong Yi Xiong")
						.push_line("Source code: https://github.com/TheRealMintd/rusty-musicbot")
						.push_line("License: AGPLv3"),
				)
			})
		})
		.await?;

	Ok(())
}
