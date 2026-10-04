/// This module defines the EncounterFilter, which is a specific implementation of the Filter trait
/// that allow filtering specific encounters tyoe, name or date.
use parser::core::encounter::Encounter;

use crate::filter::Filter;

/// Filter that selects encounters based on their name.
pub struct EncounterFilter {
    name: Option<String>,
}

impl EncounterFilter {
    /// Creates a new EncounterFilter with the given name criteria.
    ///
    /// # Arguments
    /// - `name`: Optional criteria for the encounter name. If None, all names are considered.
    ///
    /// # Return
    /// A new instance of EncounterFilter configured with the specified criteria.
    pub fn new(name: Option<String>) -> Self {
        EncounterFilter { name }
    }
}

/// Implementation of the Filter trait for EncounterFilter,
/// allowing it to be used in a filter chain to process encounters based on encounter name criteria.
impl Filter<Vec<Encounter>> for EncounterFilter {
    fn apply(&self, encounters: &Vec<Encounter>) -> Vec<Encounter> {
        encounters
            .iter()
            .filter(|e| {
                if let Some(name) = &self.name {
                    e.title.contains(name)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    fn encounter_title(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
