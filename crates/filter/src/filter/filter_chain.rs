use parser::core::encounter::Encounter;

use crate::filter::Filter;

pub struct FilterChain {
    filters: Vec<Box<dyn Filter<Vec<Encounter>> + Send + Sync>>,
}

impl std::fmt::Debug for FilterChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FilterChain({} filter(s))", self.filters.len())
    }
}

/// A filter chain that allows applying multiple filters sequentially to a list of encounters.
impl FilterChain {
    /// Creates a new, empty FilterChain.
    ///
    /// # Return
    /// A new instance of FilterChain with no filters added.
    pub fn new() -> Self {
        FilterChain { filters: vec![] }
    }

    /// Adds a new filter to the chain.
    ///
    /// # Arguments
    /// - `filter`: An implementation of the Filter trait that operates on a vector of encounters.
    ///
    /// # Return
    /// The FilterChain instance with the new filter added, allowing for method chaining.
    pub fn add(mut self, filter: impl Filter<Vec<Encounter>> + Send + Sync + 'static) -> Self {
        self.filters.push(Box::new(filter));
        self
    }
    /// Returns the encounter title from the first filter in the chain that provides one.
    pub fn encounter_title(&self) -> Option<&str> {
        self.filters.iter().find_map(|f| f.encounter_title())
    }

    pub fn price_min(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| f.price_min())
    }

    pub fn price_max(&self) -> Option<f64> {
        self.filters.iter().find_map(|f| f.price_max())
    }

    pub fn seat_category(&self) -> Option<&str> {
        self.filters.iter().find_map(|f| f.seat_category())
    }

    pub fn seat_bloc(&self) -> Option<&str> {
        self.filters.iter().find_map(|f| f.seat_bloc())
    }

    pub fn seat_row(&self) -> Option<&str> {
        self.filters.iter().find_map(|f| f.seat_row())
    }

    pub fn side_by_side(&self) -> Option<usize> {
        self.filters.iter().find_map(|f| f.side_by_side())
    }
}

impl Filter<Vec<Encounter>> for FilterChain {
    /// Applies the filter chain to a list of encounters, sequentially applying each filter in the chain.
    ///
    /// # Arguments
    /// - `encounters`: A reference to a vector of encounters to be filtered.
    ///
    /// # Return
    /// A vector of encounters that have passed through all filters in the chain.
    fn apply(&self, encounters: &Vec<Encounter>) -> Vec<Encounter> {
        self.filters
            .iter()
            .fold(encounters.clone(), |acc, filter| filter.apply(&acc))
    }
}
