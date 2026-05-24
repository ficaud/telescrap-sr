#![allow(unused)]
use admin_panel::interface::admin_panel;
use parser::{
    controller::encounter_store::StoreEncounters,
    core::{
        club::ClubType,
        encounter::{Encounter, MatchNature},
    },
    interface::storage::EncounterStore,
};
use scanner::{controller::notify::Notify, core::scan};
use scanner::interface::runner::ScannerHandle;
use telegram_notifier::TelegramNotifier;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Load Telegram bot token and chat ID from environment variables
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN non défini dans .env");
    let chat_id: i64 = std::env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID non défini dans .env")
        .parse()
        .expect("TELEGRAM_CHAT_ID doit être un entier");

    // ------- Step 0 : Initialize components -------
    // Telegram notifier configuration with environment variables
    let notifier = TelegramNotifier::new(bot_token, chat_id, env!("CARGO_PKG_VERSION"));
    // Scanner configuration (interval, club, match type, filters)
    let scan_config = ScannerHandle::configure();

    // ------- Step 1 : Notify startup and create config channel -------
    // notifier.notify_state(scan_config.clone());

    // ------- Step 2 : Create the watch channel -------
    // Create the watch channel — config_tx allows sending config updates at runtime
    let (config_tx, config_rx) = watch::channel(scan_config);


    // ------- Step 3 : Start scanner and admin panel task -------
    // Start the scanner with the config receiver and notifier
    let _handle = ScannerHandle::start(config_rx, notifier);
    // Start the admin panel web server in a separate task (owns config_tx)
    tokio::spawn(admin_panel::run(config_tx));

    // ------- Step 4 : Wait for shutdown signal (Ctrl+C) -------
    tokio::signal::ctrl_c().await.unwrap();
}
