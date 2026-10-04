use crate::core::bot_state::BotState;
use chrono::Utc;
use chrono_tz::Europe::Paris;
use parser::interface::storage::BotStateStore;
use scanner::{controller::notify::Notify, core::scan::ScanConfig};
use teloxide::{
    prelude::*,
    types::{InputFile, ParseMode},
};

/// This module defines the `TelegramNotifier` struct, which implements the `Notify` trait to send notifications to a Telegram chat.
#[derive(Clone)]
pub struct TelegramNotifier {
    bot_token: String,
    chat_id: i64,
    version: String,
}

impl TelegramNotifier {
    /// Creates a new instance of `TelegramNotifier` with the specified bot token, chat ID and app version.
    pub fn new(bot_token: String, chat_id: i64, version: &str) -> Self {
        Self {
            bot_token,
            chat_id,
            version: version.to_string(),
        }
    }

    /// Sends or updates the bot's status message in the Telegram chat, including the current version and startup time.
    pub fn notify_state(&self, scan_config: ScanConfig) {
        let state_db = BotStateStore::open("state.db").expect("Impossible d'ouvrir state.db");
        let now = Utc::now().with_timezone(&Paris);
        let state = BotState {
            pinned_message_id: state_db.get_pinned_message_id().ok().flatten(),
        };
        let header = format!(
            "<b>[telescrap-sr] <a href=\"https://github.com/Thejulfi/telescrap-sr/blob/main/CHANGELOG.md\">v{}</a></b>",
            self.version
        );

        // let mode = match scan_config.mode {
        //     ScanMode::PassiveScan => "Passif",
        //     ScanMode::AggressiveScan => "Agressif",
        // };
        let interval_min = scan_config.interval / 60;
        let interval_sec = scan_config.interval % 60;
        let interval_str = if interval_min > 0 {
            format!("{}m{}s", interval_min, interval_sec)
        } else {
            format!("{}s", interval_sec)
        };

        let config_lines = vec![
            format!("-  Club visé : {}", scan_config.club.name),
            format!("-  Intervalle : {}", interval_str),
            format!(
                "-  Filtres : {}",
                if scan_config.filter_chain.is_some() {
                    "Oui"
                } else {
                    "Non"
                }
            ),
            format!(
                "-  Aperçu : {}",
                if scan_config.is_preview { "Oui" } else { "Non" }
            ),
        ];

        let config_block = config_lines.join("\n");
        let full_message = format!(
            "{}\n\n{}\n\n<i>🕐 En cours depuis le {}</i>",
            header,
            config_block,
            now.format("%d/%m/%Y à %H:%M:%S")
        );

        if let Some(id) = state.pinned_message_id {
            self.edit_message(id, &full_message);
        } else if let Some(new_id) = self.send_and_pin(&full_message) {
            state_db.set_pinned_message_id(new_id).ok();
        }
    }
}

impl Notify for TelegramNotifier {
    fn send(&self, message: &str) {
        let bot = Bot::new(&self.bot_token);
        let chat_id = ChatId(self.chat_id);
        let message = message.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bot.send_message(chat_id, message)
                    .parse_mode(ParseMode::Html)
                    .await
                    .ok();
            });
        });
    }

    fn send_photo(&self, photo_url: &str, caption: &str) {
        let bot = Bot::new(&self.bot_token);
        let chat_id = ChatId(self.chat_id);
        let caption = caption.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Ok(url) = reqwest::Url::parse(&photo_url) {
                    bot.send_photo(chat_id, InputFile::url(url))
                        .caption(caption)
                        .parse_mode(ParseMode::Html)
                        .await
                        .ok();
                }
            });
        });
    }

    fn edit_message(&self, message_id: i32, message: &str) {
        let bot = Bot::new(&self.bot_token);
        let chat_id = ChatId(self.chat_id);
        let message = message.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bot.edit_message_text(chat_id, teloxide::types::MessageId(message_id), message)
                    .parse_mode(ParseMode::Html)
                    .await
                    .ok();
            });
        });
    }

    fn send_and_pin(&self, message: &str) -> Option<i32> {
        let bot = Bot::new(&self.bot_token);
        let chat_id = ChatId(self.chat_id);
        let message = message.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(msg) = bot
                    .send_message(chat_id, message)
                    .parse_mode(ParseMode::Html)
                    .await
                    .ok()
                {
                    bot.pin_chat_message(chat_id, msg.id).await.ok();
                    Some(msg.id.0)
                } else {
                    None
                }
            })
        })
    }
}
