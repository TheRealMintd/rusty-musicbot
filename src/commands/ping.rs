use serenity::{
	framework::standard::{macros::command, CommandResult},
	model::channel::Message,
	prelude::*,
};

#[command]
/// Causes the bot to reply with "Pong!"
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
	msg.channel_id.say(&ctx.http, "Pong!").await?;

	Ok(())
}
