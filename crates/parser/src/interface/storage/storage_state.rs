use super::redb::StorageError;
use redb::{Database, ReadableDatabase, TableDefinition};
/// This module implements the `BotStateStore` struct, which provides persistent storage for the bot's runtime state
/// using a `redb` embedded database. Currently, the state consists of the pinned Telegram message ID,
/// which is updated at each startup to reflect the current version.
use std::path::Path;

const STATE_TABLE: TableDefinition<&str, i32> = TableDefinition::new("state");

const KEY_PINNED_MSG_ID: &str = "pinned_message_id";

/// Persistent store for bot state, backed by a `redb` database.
pub struct BotStateStore {
    db: Database,
}

impl BotStateStore {
    /// Opens the state database at the specified path, creating it if it doesn't exist,
    /// and ensures the state table is initialized.
    ///
    /// # Arguments
    /// * `path` - The file path where the state database should be created or opened
    ///
    /// # Returns
    /// A `Result` containing the `BotStateStore` instance if successful, or a `StorageError` if an error occurs.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        txn.open_table(STATE_TABLE)?;
        txn.commit()?;
        Ok(Self { db })
    }

    /// Sets the pinned message ID in the state database, which is used to track the Telegram message that should be edited on bot restart.
    ///
    /// # Arguments
    /// * `id` - The Telegram message ID to store as the pinned message ID
    ///
    /// # Returns
    /// A `Result` indicating success or containing a `StorageError` if an error occurs
    pub fn set_pinned_message_id(&self, id: i32) -> Result<(), StorageError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(STATE_TABLE)?;
            table.insert(KEY_PINNED_MSG_ID, id)?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Retrieves the pinned message ID from the state database, if it exists.
    ///
    /// # Returns
    /// A `Result` containing an `Option<i32>` with the pinned message ID if it exists, or `None` if it does not,
    /// or a `StorageError` if an error occurs.
    pub fn get_pinned_message_id(&self) -> Result<Option<i32>, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(STATE_TABLE)?;
        Ok(table.get(KEY_PINNED_MSG_ID)?.map(|v| v.value()))
    }
}
