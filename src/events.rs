use std::sync::Arc;

use serenity::{async_trait, model::id::GuildId};
use songbird::{
	Event, EventContext, EventHandler as VoiceEventHandler, Songbird,
};
use tracing::error;

pub(crate) struct TrackEnd {
	pub guild_id: GuildId,
	pub manager: Arc<Songbird>,
}

#[async_trait]
impl VoiceEventHandler for TrackEnd {
	async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
		if let EventContext::Track(_) = ctx {
			let handler_lock = self.manager.get(self.guild_id)?;
			let queue_empty = handler_lock.lock().await.queue().is_empty();

			if queue_empty {
				if let Err(e) = self.manager.remove(self.guild_id).await {
					error!(
						"Could not leave {}'s voice channel: {:?}",
						self.guild_id, e
					);
				}
			}
		}

		None
	}
}
