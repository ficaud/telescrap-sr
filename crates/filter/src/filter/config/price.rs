/// This module defines the PriceFilter, which is a specific implementation of the Filter trait
/// that allows filtering encounters based on its seat's price.
use parser::core::encounter::Encounter;

use crate::filter::Filter;

/// Filter that selects encounters based on the position of available seats.
/// It can filter by specific seat composition (category, bloc, row)
/// and/or by a minimum number of consecutive seats.
pub struct PriceFilter {
    min_price: Option<f64>,
    max_price: Option<f64>,
}

impl PriceFilter {
    /// Creates a new PriceFilter with the given minimum and maximum price.
    ///
    /// # Arguments
    /// - `min_price`: Optional minimum price. If None, no minimum is enforced.
    /// - `max_price`: Optional maximum price. If None, no maximum is enforced.
    ///
    /// # Return
    /// A new instance of PriceFilter configured with the specified criteria.
    pub fn new(min_price: Option<f64>, max_price: Option<f64>) -> Self {
        PriceFilter {
            min_price,
            max_price,
        }
    }
}

/// Implementation of the Filter trait for PriceFilter,
/// allowing it to be used in a filter chain to process encounters based on price criteria.
impl Filter<Vec<Encounter>> for PriceFilter {
    fn price_min(&self) -> Option<f64> {
        self.min_price
    }

    fn price_max(&self) -> Option<f64> {
        self.max_price
    }

    /// Applies the price filter to a list of encounters,
    /// returning only those encounters that have seats within the specified price range.
    ///
    /// # Arguments
    /// - `encounters`: A reference to a vector of encounters to be filtered.
    ///
    /// # Return
    /// A vector of encounters that match the price criteria defined in the filter.
    fn apply(&self, encounters: &Vec<Encounter>) -> Vec<Encounter> {
        encounters
            .iter()
            .filter_map(|encounter| {
                let matching_seats: Vec<_> = encounter
                    .seats
                    .as_ref()?
                    .iter()
                    .filter(|seat| {
                        let price_str: String = seat
                            .price
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
                            .collect::<String>()
                            .replace(',', ".");
                        price_str.parse::<f64>().map_or(false, |price| {
                            (self.min_price.is_none() || price >= self.min_price.unwrap())
                                && (self.max_price.is_none() || price <= self.max_price.unwrap())
                        })
                    })
                    .cloned()
                    .collect();

                if matching_seats.is_empty() {
                    return None;
                }

                let mut encounter = encounter.clone();
                encounter.seats = Some(matching_seats);
                Some(encounter)
            })
            .collect()
    }
}
