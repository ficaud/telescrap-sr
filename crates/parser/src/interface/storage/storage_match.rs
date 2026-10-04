use super::redb::StorageError;
use crate::{
    controller::encounter_store::{EncounterRecord, StoreEncounters},
    core::encounter::Encounter,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
/// This module implements the `StoreEncounters` trait using the `redb` embedded database.
/// It defines the `EncounterStore` struct, which provides methods for synchronizing parsed encounters with
/// the store, retrieving active resale links, looking up records by stable ID, and retrieving all records from the store.
use std::path::Path;
// Encounters are stored in a single table with `stable_id` as the key and the JSON-serialized `EncounterRecord` as the value.
const ENCOUNTERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("encounters");

/// Helper function to create a stable ID for an encounter based on its title and date, which is used as the key in the database.
fn make_stable_id(title: &str, date: &str) -> String {
    format!("{title}|{date}")
}

/// Encounter store structure that uses `redb` for persistent storage of encounter records,
/// implementing the `StoreEncounters` trait to provide methods for synchronizing parsed encounters with the store,
/// retrieving active resale links, looking up records by stable ID, and retrieving all records from the store.
pub struct EncounterStore {
    db: Database,
}

impl EncounterStore {
    /// Opens a `redb` database at the specified path and initializes the encounters table
    /// if it does not already exist.
    ///
    /// # Arguments
    /// * `path` - The file path where the `redb` database should be created or opened
    /// # Returns
    /// A `Result` containing an instance of `EncounterStore` if the database was successfully opened and initialized,
    /// or a `StorageError` if there was an error during the process.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        txn.open_table(ENCOUNTERS_TABLE)?;
        txn.commit()?;
        Ok(Self { db })
    }
}

impl StoreEncounters for EncounterStore {
    /// Synchronizes a parsed `Encounter` with the store by inserting or updating the corresponding `EncounterRecord` in the database.
    /// If the `Encounter` has a `resale_link`, it will be inserted or updated with `resale_active = true`.
    /// If the `Encounter` does not have a `resale_link`, the existing record (if any) will be updated to set `resale_active = false`
    /// while keeping the existing URL.
    ///
    /// # Arguments
    /// * `encounter` - The `Encounter` instance to be synchronized with the store
    /// # Returns
    /// A `Result` indicating success or containing a `StorageError` if there was an error during the synchronization process.
    fn upsert(&self, encounter: &Encounter) -> Result<(), String> {
        (|| -> Result<(), StorageError> {
            let stable_id = make_stable_id(&encounter.title, &encounter.date);
            let txn = self.db.begin_write()?;
            {
                let mut table = txn.open_table(ENCOUNTERS_TABLE)?;

                if let Some(resale_link) = &encounter.resale_link {
                    let record = EncounterRecord {
                        stable_id: stable_id.clone(),
                        title: encounter.title.clone(),
                        date: encounter.date.clone(),
                        club_type: format!("{:?}", encounter.club_type),
                        resale_link: resale_link.clone(),
                        resale_active: true,
                    };
                    let json = serde_json::to_string(&record)?;
                    table.insert(stable_id.as_str(), json.as_str())?;
                } else {
                    let existing_json = table
                        .get(stable_id.as_str())?
                        .map(|guard| guard.value().to_owned());

                    if let Some(existing_str) = existing_json {
                        let mut record: EncounterRecord = serde_json::from_str(&existing_str)?;
                        if record.resale_active {
                            record.resale_active = false;
                            let json = serde_json::to_string(&record)?;
                            table.insert(stable_id.as_str(), json.as_str())?;
                        }
                    }
                }
            }
            txn.commit()?;
            Ok(())
        })()
        .map_err(|e| e.to_string())
    }

    /// Retrieves all records from the store that have an active resale link.
    ///
    /// # Returns
    /// A `Result` containing a vector of `EncounterRecord` instances with active resale links if the retrieval was successful,
    /// or a `StorageError` if there was an error during the retrieval process.
    fn get_active_resale_links(&self) -> Result<Vec<EncounterRecord>, String> {
        (|| -> Result<Vec<EncounterRecord>, StorageError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            let mut result = Vec::new();
            for entry in table.iter()? {
                let item = entry?;
                if let Ok(record) = serde_json::from_str::<EncounterRecord>(item.1.value()) {
                    if record.resale_active {
                        result.push(record);
                    }
                }
            }
            Ok(result)
        })()
        .map_err(|e| e.to_string())
    }

    /// Looks up a single record in the store by its stable ID, which is derived from the encounter's title and date.
    ///
    /// # Arguments
    /// * `title` - The title of the encounter to look up
    /// * `date` - The date of the encounter to look up
    /// # Returns
    /// A `Result` containing an `Option<EncounterRecord>` if the lookup was successful,
    /// where `Some(EncounterRecord)` is returned if a record with the specified title and date exists, and `None` is returned if no such record exists.
    /// If there was an error during the lookup process, a `StorageError` is returned
    fn get_by_stable_id(&self, title: &str, date: &str) -> Result<Option<EncounterRecord>, String> {
        (|| -> Result<Option<EncounterRecord>, StorageError> {
            let stable_id = make_stable_id(title, date);
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            match table.get(stable_id.as_str())? {
                Some(guard) => {
                    let record: EncounterRecord = serde_json::from_str(guard.value())?;
                    Ok(Some(record))
                }
                None => Ok(None),
            }
        })()
        .map_err(|e| e.to_string())
    }

    /// Retrieves all records from the store, regardless of their resale link status.
    ///
    /// # Returns
    /// A `Result` containing a vector of all `EncounterRecord` instances in the store if the retrieval was successful,
    /// or a `StorageError` if there was an error during the retrieval process.
    fn get_all(&self) -> Result<Vec<EncounterRecord>, String> {
        (|| -> Result<Vec<EncounterRecord>, StorageError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            let mut result = Vec::new();
            for entry in table.iter()? {
                let item = entry?;
                if let Ok(record) = serde_json::from_str::<EncounterRecord>(item.1.value()) {
                    result.push(record);
                }
            }
            Ok(result)
        })()
        .map_err(|e| e.to_string())
    }
}
