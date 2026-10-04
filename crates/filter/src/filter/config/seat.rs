/// This module defines the SeatPositionFilter, which is a specific implementation of the Filter trait
/// that allows filtering encounters based on the position of available seats.
///
/// It can filter by specific seat composition (category, bloc, row) and/or by a minimum number of consecutive seats.
use parser::core::{
    encounter::Encounter,
    seat::{Seat, SeatComposition},
};

use crate::filter::Filter;

/// Filter that selects encounters based on the position of available seats.
/// It can filter by specific seat composition (category, bloc, row)
/// and/or by a minimum number of consecutive seats.
pub struct SeatPositionFilter {
    composition: Option<SeatComposition>,
    min_consecutive: Option<usize>,
}

impl SeatPositionFilter {
    /// Creates a new SeatPositionFilter with the given composition criteria and minimum consecutive seats.
    ///
    /// # Arguments
    /// - `composition`: Optional criteria for seat composition (category, bloc, row). If None, all compositions are considered.
    /// - `min_consecutive`: Optional minimum number of consecutive seats required. If None, no minimum is enforced.
    ///
    /// # Return
    /// A new instance of SeatPositionFilter configured with the specified criteria.
    pub fn new(composition: Option<SeatComposition>, min_consecutive: Option<usize>) -> Self {
        SeatPositionFilter {
            composition,
            min_consecutive,
        }
    }

    /// Checks if a given seat matches the composition criteria of the filter.
    ///
    /// # Arguments
    /// - `seat`: The seat to check against the filter's composition criteria.
    ///
    /// # Return
    /// `true` if the seat matches the composition criteria or if no composition criteria is set; `false` otherwise.
    pub fn match_seat_composition(&self, seat: &Seat) -> bool {
        match &self.composition {
            None => true,
            Some(comp) => {
                let c = &seat.seat_info.composition;
                (comp.category.is_empty()
                    || c.category
                        .to_lowercase()
                        .contains(&comp.category.to_lowercase()))
                    && (comp.bloc.is_empty()
                        || c.bloc.to_lowercase().contains(&comp.bloc.to_lowercase()))
                    && (comp.row.is_empty() || c.row.to_lowercase() == comp.row.to_lowercase())
                    && (comp.seat_number == 0 || c.seat_number == comp.seat_number)
            }
        }
    }

    /// Groups a list of seats into sublists of adjacent seats based on their row and seat numbers.
    ///
    /// # Arguments
    /// - `seats`: A slice of seats to be grouped into consecutive groups.
    ///
    /// # Return
    /// A vector of vectors, where each inner vector contains seats from the same row whose
    /// seat numbers differ by 2, matching the current adjacency rule used by the venue.
    pub fn consecutive_seats(seats: &[Seat]) -> Vec<Vec<Seat>> {
        if seats.is_empty() {
            return vec![];
        }

        let mut sorted: Vec<&Seat> = seats.iter().collect();
        sorted.sort_by(|left, right| {
            left.seat_info
                .composition
                .row
                .cmp(&right.seat_info.composition.row)
                .then_with(|| {
                    left.seat_info
                        .composition
                        .seat_number
                        .cmp(&right.seat_info.composition.seat_number)
                })
        });

        let mut groups: Vec<Vec<Seat>> = vec![];
        let mut current = vec![(*sorted[0]).clone()];

        for i in 1..sorted.len() {
            let current_row = &sorted[i].seat_info.composition.row;
            let previous_row = &sorted[i - 1].seat_info.composition.row;
            let current_seat = sorted[i].seat_info.composition.seat_number;
            let previous_seat = sorted[i - 1].seat_info.composition.seat_number;

            if current_row == previous_row && current_seat == previous_seat + 2 {
                current.push((*sorted[i]).clone());
            } else {
                groups.push(current);
                current = vec![(*sorted[i]).clone()];
            }
        }
        groups.push(current);
        groups
    }
}

/// Implementation of the Filter trait for SeatPositionFilter,
/// allowing it to be used in a filter chain to process encounters based on seat position criteria.
impl Filter<Vec<Encounter>> for SeatPositionFilter {
    fn seat_category(&self) -> Option<&str> {
        self.composition
            .as_ref()
            .map(|c| c.category.as_str())
            .filter(|s| !s.is_empty())
    }

    fn seat_bloc(&self) -> Option<&str> {
        self.composition
            .as_ref()
            .map(|c| c.bloc.as_str())
            .filter(|s| !s.is_empty())
    }

    fn seat_row(&self) -> Option<&str> {
        self.composition
            .as_ref()
            .map(|c| c.row.as_str())
            .filter(|s| !s.is_empty())
    }

    fn side_by_side(&self) -> Option<usize> {
        self.min_consecutive
    }

    /// Applies the seat position filter to a list of encounters,
    /// returning only those encounters that have seats matching the specified composition
    /// and minimum consecutive criteria.
    ///
    /// # Arguments
    /// - `encounters`: A reference to a vector of encounters to be filtered.
    ///
    /// # Return
    /// A vector of encounters that match the seat position criteria defined in the filter.
    fn apply(&self, encounters: &Vec<Encounter>) -> Vec<Encounter> {
        let min = self.min_consecutive.unwrap_or(1);
        encounters
            .iter()
            .filter_map(|encounter| {
                if let Some(seats) = &encounter.seats {
                    // Filter matchin seats based on composition criteria
                    let matching_seats: Vec<_> = seats
                        .iter()
                        .filter(|s| self.match_seat_composition(s))
                        .cloned()
                        .collect();

                    if matching_seats.is_empty() {
                        return None;
                    }

                    // Create a list of compositions for the matching seats in the current match
                    // Check if there are enough consecutive seats among the matching ones
                    let has_enough = Self::consecutive_seats(&matching_seats)
                        .into_iter()
                        .find(|group| group.len() >= min);

                    if let Some(consecutive_seats) = has_enough {
                        let mut result = encounter.clone();
                        result.seats = Some(consecutive_seats);
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
