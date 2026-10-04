use crate::core::encounter::Encounter;
/// This module defines the EncounterRecord struct, which represents a stable representation of an encounter as stored in the database,
/// and the StoreEncounters trait, which provides methods for synchronizing parsed encounters with the store,
/// retrieving active resale links, looking up records by stable ID, and retrieving all records from the store.
use serde::{Deserialize, Serialize};

/// Stable representation of an encounter as stored in the database.
///
/// The `stable_id` is derived from `title|date` and acts as the primary key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterRecord {
    pub stable_id: String,
    pub title: String,
    pub date: String,
    pub club_type: String,
    /// The resale URL captured when it was available.
    pub resale_link: String,
    /// `true` while the link was visible on the last full-list parse.
    pub resale_active: bool,
}

pub trait StoreEncounters {
    /// Synchronises one parsed `Encounter` with the store.
    ///
    /// - `resale_link` present  → insert or update the record, `resale_active = true`.
    /// - `resale_link` absent   → if a record already exists, mark `resale_active = false`
    ///                            but **keep the URL** so the scanner can still poll it.
    fn upsert(&self, encounter: &Encounter) -> Result<(), String>;

    /// Returns all records whose resale link is currently considered active.
    fn get_active_resale_links(&self) -> Result<Vec<EncounterRecord>, String>;

    /// Looks up a single record by match title and date.
    fn get_by_stable_id(&self, title: &str, date: &str) -> Result<Option<EncounterRecord>, String>;

    /// Returns all records in the store.
    fn get_all(&self) -> Result<Vec<EncounterRecord>, String>;
}
