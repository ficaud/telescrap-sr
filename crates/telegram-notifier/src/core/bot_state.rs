/// Represents the persistent runtime state of the bot.
#[derive(Debug, Clone)]
pub struct BotState {
    /// ID of the pinned Telegram status message, used to edit it on restart instead of creating a new one.
    pub pinned_message_id: Option<i32>,
}

// impl BotState {
//     // pub fn new() -> Self {
//     //     Self { pinned_message_id: None, started_at: None, version: env!("CARGO_PKG_VERSION").to_string() }
//     // }
// }
