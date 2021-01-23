use std::collections::HashSet;

use serenity::{
	client::Context,
	framework::standard::{
		help_commands::with_embeds, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
	},
	model::prelude::*,
};

#[help]
async fn help(
	ctx: &Context,
	msg: &Message,
	args: Args,
	help_options: &'static HelpOptions,
	groups: &[&'static CommandGroup],
	owners: HashSet<UserId>,
) -> CommandResult {
	with_embeds(ctx, msg, args, help_options, groups, owners).await;
	Ok(())
}
