use parser::core::encounter::Encounter;

/// A trait for filtering encounter bas on specific criteria.
///
/// Implementors of this trait can define their own filtering logic to return a subset of encounters that match certain conditions.
pub trait Filter<T> {
    fn apply(&self, item: &T) -> Vec<Encounter>;

    /// Returns the encounter title criteria for this filter, if any.
    fn encounter_title(&self) -> Option<&str> {
        None
    }

    /// Returns the minimum price criteria for this filter, if any.
    fn price_min(&self) -> Option<f64> {
        None
    }

    /// Returns the maximum price criteria for this filter, if any.
    fn price_max(&self) -> Option<f64> {
        None
    }

    /// Returns the seat category criteria for this filter, if any.
    fn seat_category(&self) -> Option<&str> {
        None
    }

    /// Returns the seat bloc criteria for this filter, if any.
    fn seat_bloc(&self) -> Option<&str> {
        None
    }

    /// Returns the seat row criteria for this filter, if any.
    fn seat_row(&self) -> Option<&str> {
        None
    }

    /// Returns the minimum consecutive seats criteria for this filter, if any.
    fn side_by_side(&self) -> Option<usize> {
        None
    }
}
