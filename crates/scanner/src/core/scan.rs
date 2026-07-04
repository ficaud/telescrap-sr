/// This module defines the core structures and logic for scanning encounters based on specified configurations and filters.
use std::sync::Arc;
use std::time::SystemTime;
use filter::filter::filter_chain::FilterChain;
use parser::core::{
    club::Club,
    encounter::{Encounter, MatchNature},
};

/// Represents the mode of scanning, which can be either passive or aggressive.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    PassiveScan,
    AggressiveScan,
}

/// Represents the configuration for scanning encounters, including the mode, interval, club, match nature, and optional filters.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub mode: ScanMode,
    pub interval: u64,
    pub club: Club,
    pub nature: MatchNature,
    // pub match_title: Option<String>,
    pub is_preview: bool,
    pub proxy_enabled: bool,
    pub filter_chain: Option<Arc<FilterChain>>,
}

/// Represents the result of a scan, containing the list of encounters found and the timestamp of when the scan was performed.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl Default for ScanConfig {
    /// Loads `ScanConfig` from `config_scan.json` at the working directory.
    /// Panics with a descriptive message if the file is missing or invalid.
    fn default() -> Self {
        crate::core::config_file::load_from_file("config_scan.json")
            .unwrap_or_else(|e| panic!("❌ Impossible de charger config_scan.json : {}", e))
    }
}

impl ScanResult {
    /// Creates a new `ScanResult` with the specified list of encounters.
    ///
    /// # Arguments
    /// * `encounters` - A vector of `Encounter` instances representing the results of the scan.
    /// 
    /// # Returns
    /// A new instance of `ScanResult` initialized with the provided encounters and the current timestamp.
    pub fn new(encounters: Vec<Encounter>) -> Self {
        Self { encounters, scanned_at: SystemTime::now() }
    }
}