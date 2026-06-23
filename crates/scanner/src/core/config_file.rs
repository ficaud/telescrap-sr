/// This module handles serialization and deserialization of a `config_scan.json` file into a `ScanConfig` or from a `ScanConfig` back to JSON.
///
/// The JSON format uses plain string enums and a flat list of filter descriptors,
/// keeping the domain types (`ScanConfig`, `FilterChain`, `Club`…) free of serde concerns.
/// Conversion is done via `TryFrom<ScanConfigRaw>`.
///
/// # `config_scan.json` reference
///
/// Place this file at the root of the workspace (next to `Cargo.toml`).
///
/// ## Top-level fields
///
/// | Field          | Type              | Values                                      | Description                                      |
/// |----------------|-------------------|---------------------------------------------|--------------------------------------------------|
/// | `mode`         | string            | `"Passive"` · `"Aggressive"`                | Passive: notify only. Aggressive: auto add-to-cart. |
/// | `interval`     | integer (seconds) | e.g. `30`, `60`, `120`                      | Delay between two consecutive scans.             |
/// | `club`         | string            | `"StadeRochelais"` · `"UnionBordeauxBegles"`| Club whose ticketing site is scraped.            |
/// | `nature`       | string            | `"Rugby"` · `"Basketball"` · `"Other"`      | Match category to look for.                      |
/// | `is_preview`   | boolean           | `true` · `false`                            | Fetch seat preview images before notifying.      |
/// | `filter_chain` | array or `null`   | see filter types below                      | Ordered list of filters applied to results.      |
///
/// ## Filter types (`filter_chain` entries)
///
/// Every entry **must** have a `"type"` field that selects the filter kind.
/// All other fields are optional (`null` means "no constraint").
///
/// ### `"Price"` — keep seats within a price range
/// ```json
/// { "type": "Price", "min": 20.0, "max": 80.0 }
/// ```
/// | Field | Type           | Description                          |
/// |-------|----------------|--------------------------------------|
/// | `min` | float or `null`| Minimum seat price (inclusive).      |
/// | `max` | float or `null`| Maximum seat price (inclusive).      |
///
/// ### `"Encounter"` — keep only encounters whose title contains a substring
/// ```json
/// { "type": "Encounter", "name": "STADE ROCHELAIS" }
/// ```
/// | Field  | Type            | Description                                         |
/// |--------|-----------------|-----------------------------------------------------|
/// | `name` | string or `null`| Case-sensitive substring matched against the title. |
///
/// ### `"Seat"` — filter by seat location and/or minimum consecutive count
/// ```json
/// { "type": "Seat", "category": "Tribune", "bloc": "B", "row": "12", "min_consecutive": 2 }
/// ```
/// | Field             | Type              | Description                                                  |
/// |-------------------|-------------------|--------------------------------------------------------------|
/// | `category`        | string or `null`  | Seat category (partial, case-insensitive match).             |
/// | `bloc`            | string or `null`  | Bloc identifier (partial, case-insensitive match).           |
/// | `row`             | string or `null`  | Row label (exact, case-insensitive match).                   |
/// | `min_consecutive` | integer or `null` | Minimum number of adjacent seats required in the same row.   |
///
/// ## Full example
///
/// ```json
/// {
///   "mode": "Aggressive",
///   "interval": 60,
///   "club": "StadeRochelais",
///   "nature": "Rugby",
///   "is_preview": false,
///   "filter_chain": [
///     {
///       "type": "Encounter",
///       "name": "STADE ROCHELAIS / STADE FRANÇAIS"
///     },
///     {
///       "type": "Price",
///       "min": 10.0,
///       "max": 50.0
///     },
///     {
///       "type": "Seat",
///       "category": null,
///       "bloc": null,
///       "row": null,
///       "min_consecutive": 2
///     }
///   ]
/// }
/// ```
use std::sync::Arc;

use filter::filter::{
    config::{encounter::EncounterFilter, price::PriceFilter, seat::SeatPositionFilter},
    filter_chain::FilterChain,
};
use parser::core::{
    club::{Club, ClubType},
    encounter::MatchNature,
    seat::SeatComposition,
};
use serde::{Deserialize, Serialize};

use crate::core::scan::{ScanConfig, ScanMode};

// ---------------------------------------------------------------------------
// Raw DTOs — only used for JSON deserialization
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub enum ScanModeRaw {
    Passive,
    Aggressive,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ClubRaw {
    StadeRochelais,
    UnionBordeauxBegles,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NatureRaw {
    Rugby,
    Basketball,
    Other,
}

/// One filter entry in the JSON `filter_chain` array.
/// The `"type"` field acts as the discriminant tag.
///
/// Example entries:
/// ```json
/// { "type": "Price",     "min": 10.0, "max": 80.0 }
/// { "type": "Encounter", "name": "STADE ROCHELAIS" }
/// { "type": "Seat",      "category": "Tribune", "min_consecutive": 2 }
/// ```
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum FilterDescriptor {
    Price {
        min: Option<f64>,
        max: Option<f64>,
    },
    Encounter {
        name: Option<String>,
    },
    Seat {
        category: Option<String>,
        bloc: Option<String>,
        row: Option<String>,
        min_consecutive: Option<usize>,
    },
}

/// Full raw representation of `config_scan.json`.
///
/// Example file:
/// ```json
/// {
///   "mode": "Aggressive",
///   "interval": 60,
///   "club": "StadeRochelais",
///   "nature": "Rugby",
///   "is_preview": false,
///   "filter_chain": [
///     {
///       "type": "Encounter",
///       "name": "STADE ROCHELAIS / STADE FRANÇAIS"
///     },
///     {
///       "type": "Price",
///       "min": 10.0,
///       "max": 50.0
///     },
///     {
///       "type": "Seat",
///       "category": null,
///       "bloc": null,
///       "row": null,
///       "min_consecutive": 2
///     }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct ScanConfigRaw {
    pub mode: ScanModeRaw,
    pub interval: u64,
    pub club: ClubRaw,
    pub nature: NatureRaw,
    pub is_preview: bool,
    pub proxy_enabled: bool,
    pub filter_chain: Option<Vec<FilterDescriptor>>,
}

// ---------------------------------------------------------------------------
// Conversion into domain types
// ---------------------------------------------------------------------------
impl TryFrom<ScanConfig> for ScanConfigRaw {
    type Error = String;

    fn try_from(config: ScanConfig) -> Result<Self, Self::Error> {
        let mode = match config.mode {
            ScanMode::PassiveScan => ScanModeRaw::Passive,
            ScanMode::AggressiveScan => ScanModeRaw::Aggressive,
        };

        let club = match config.club.club_type {
            ClubType::StadeRochelais => ClubRaw::StadeRochelais,
            ClubType::UnionBordeauxBegles => ClubRaw::UnionBordeauxBegles,
        };

        let nature = match config.nature {
            MatchNature::Rugby => NatureRaw::Rugby,
            MatchNature::Basketball => NatureRaw::Basketball,
            MatchNature::Other => NatureRaw::Other,
        };


        let filter_encouter = config.filter_chain.as_ref().and_then(|chain| {
            chain.encounter_title().map(|name| FilterDescriptor::Encounter { name: Some(name.to_string()) })
        });

        let filter_seat = config.filter_chain.as_ref().and_then(|chain| {
            let category = chain.seat_category().map(|s| s.to_string());
            let bloc = chain.seat_bloc().map(|s| s.to_string());
            let row = chain.seat_row().map(|s| s.to_string());
            let min_consecutive = chain.side_by_side();
            if category.is_some() || bloc.is_some() || row.is_some() || min_consecutive.is_some() {
                Some(FilterDescriptor::Seat { category, bloc, row, min_consecutive })
            } else {
                None
            }
        });

        let filter_price = config.filter_chain.as_ref().and_then(|chain| {
            if chain.price_min().is_some() || chain.price_max().is_some() {
                Some(FilterDescriptor::Price { min: chain.price_min(), max: chain.price_max() })
            } else {
                None
            }
        });

        let filter_chain = if filter_encouter.is_none() && filter_seat.is_none() && filter_price.is_none() {
            None
        } else {
            Some(vec![filter_encouter, filter_price, filter_seat].into_iter().flatten().collect())
        };



        // Note: we don't convert the filter chain back to raw descriptors here,
        // as it's only needed for writing back to JSON (see `write_to_file`).
        // This conversion is one-way (raw -> domain), so we can ignore this case.
        Ok(ScanConfigRaw {
            mode,
            interval: config.interval,
            club,
            nature,
            is_preview: config.is_preview,
            proxy_enabled: config.proxy_enabled,
            filter_chain: filter_chain,
        })
    }
}

impl TryFrom<ScanConfigRaw> for ScanConfig {
    type Error = String;

    fn try_from(raw: ScanConfigRaw) -> Result<Self, Self::Error> {
        let mode = match raw.mode {
            ScanModeRaw::Passive => ScanMode::PassiveScan,
            ScanModeRaw::Aggressive => ScanMode::AggressiveScan,
        };

        let club = match raw.club {
            ClubRaw::StadeRochelais => Club::new(
                "Stade Rochelais".to_string(),
                ClubType::StadeRochelais,
                "https://billetterie.staderochelais.com/fr".to_string(),
            ),
            ClubRaw::UnionBordeauxBegles => Club::new(
                "Union Bordeaux Bègles".to_string(),
                ClubType::UnionBordeauxBegles,
                "https://billetterie.ubbrugby.com/fr".to_string(),
            ),
        };

        let nature = match raw.nature {
            NatureRaw::Rugby => MatchNature::Rugby,
            NatureRaw::Basketball => MatchNature::Basketball,
            NatureRaw::Other => MatchNature::Other,
        };

        let filter_chain = raw.filter_chain.map(|descriptors| {
            let chain = descriptors
                .into_iter()
                .fold(FilterChain::new(), |chain, desc| match desc {
                    FilterDescriptor::Price { min, max } => chain.add(PriceFilter::new(min, max)),
                    FilterDescriptor::Encounter { name } => chain.add(EncounterFilter::new(name)),
                    FilterDescriptor::Seat {
                        category,
                        bloc,
                        row,
                        min_consecutive,
                    } => {
                        let composition = if category.is_some() || bloc.is_some() || row.is_some() {
                            Some(SeatComposition {
                                category: category.unwrap_or_default(),
                                bloc: bloc.unwrap_or_default(),
                                row: row.unwrap_or_default(),
                                seat_number: 0,
                            })
                        } else {
                            None
                        };
                        chain.add(SeatPositionFilter::new(composition, min_consecutive))
                    }
                });
            Arc::new(chain)
        });

        Ok(ScanConfig {
            mode,
            interval: raw.interval,
            club,
            nature,
            is_preview: raw.is_preview,
            proxy_enabled: raw.proxy_enabled,
            filter_chain,
        })
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Reads and parses `path` as a `config_scan.json`, returning a ready-to-use `ScanConfig`.
/// Errors are human-readable strings describing what went wrong (I/O or JSON parse).
pub fn load_from_file(path: &str) -> Result<ScanConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;
    let raw: ScanConfigRaw = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file '{}': {}", path, e))?;
    ScanConfig::try_from(raw)
}

/// Write a `ScanConfig` back to a JSON file at `path`.
pub fn write_to_file(config: &ScanConfig, path: &str) -> Result<(), String> {
    // Convert back to raw DTOs for serialization

    let raw = ScanConfigRaw::try_from(config.clone())
        .map_err(|e| format!("Failed to convert config to raw format: {}", e))?;
    let json = serde_json::to_string_pretty(&raw)
        .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write config file '{}': {}", path, e))?;
    Ok(())
}