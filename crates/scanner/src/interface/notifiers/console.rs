/// This module defines the `ConsoleNotifier` struct, which implements the `Notify` trait to send notifications to the console.
/// The `ConsoleNotifier` is a simple implementation of the `Notify` trait that prints messages to the standard output,
/// allowing for easy debugging and monitoring of the scanning process without the need for external
use crate::controller::notify::Notify;

pub struct ConsoleNotifier;

impl Notify for ConsoleNotifier {
    fn send(&self, message: &str) {
        log::info!("[NOTIF] {}\n-----------------", message);
    }

    fn send_photo(&self, photo_url: &str, caption: &str) {
        log::info!(
            "[NOTIF PHOTO] {}\n{}\n-----------------",
            caption,
            photo_url
        );
    }

    fn send_and_pin(&self, message: &str) -> Option<i32> {
        log::info!("[NOTIF PINNED] {}\n-----------------", message);
        None
    }

    fn edit_message(&self, _message_id: i32, message: &str) {
        log::info!("[NOTIF EDITED] {}\n-----------------", message);
    }
}
