/// This module defines the logic for diffing two sets of encounters to identify new and removed seats
/// or any other changes perfomed in the resale tickets got from the parser.
use parser::core::encounter::Encounter;

fn same_seat_identity(left: &parser::core::seat::Seat, right: &parser::core::seat::Seat) -> bool {
    left.actions.pack_id == right.actions.pack_id
        && left.actions.ticket_id == right.actions.ticket_id
}

/// Enumeration representing the type of difference detected between two sets of encounters, such as new seats or removed seats.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffType {
    NewSeats,
    RemovedSeats,
}

/// Represents the result of a diff operation, containing the type of difference and the encounter
/// with only the relevant seats for that difference.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diff_type: DiffType,
    pub encounter_diff_only: Encounter,
}

/// Computes the difference between two sets of encounters, identifying new and removed seats.
/// This function is used to compare the current list of encounters with a previous list to determine what has changed,
/// such as new seats becoming available or existing seats being removed.
///
/// # Arguments
/// * `previous` - A slice of `Encounter` instances representing the previous state of encounters.
/// * `current` - A slice of `Encounter` instances representing the current state of encounters.
///
/// # Returns
/// A vector of `DiffResult` instances representing the differences detected between the two sets of encounters
pub fn diff(previous: &[Encounter], current: &[Encounter]) -> Vec<DiffResult> {
    let mut results = Vec::new();

    for encounter in current {
        let prev = previous.iter().find(|e| e.title == encounter.title);
        let current_seats = encounter.seats.as_deref().unwrap_or(&[]);

        match prev {
            None => {
                if !current_seats.is_empty() {
                    results.push(DiffResult {
                        diff_type: DiffType::NewSeats,
                        encounter_diff_only: encounter.clone(),
                    });
                }
            }
            Some(p) => {
                let prev_seats = p.seats.as_deref().unwrap_or(&[]);

                let added: Vec<_> = current_seats
                    .iter()
                    .filter(|s| !prev_seats.iter().any(|ps| same_seat_identity(ps, s)))
                    .cloned()
                    .collect();

                let removed: Vec<_> = prev_seats
                    .iter()
                    .filter(|s| !current_seats.iter().any(|cs| same_seat_identity(cs, s)))
                    .cloned()
                    .collect();

                if !added.is_empty() {
                    let mut enc = encounter.clone();
                    enc.seats = Some(added);
                    results.push(DiffResult {
                        diff_type: DiffType::NewSeats,
                        encounter_diff_only: enc,
                    });
                }

                if !removed.is_empty() {
                    let mut enc = encounter.clone();
                    enc.seats = Some(removed);
                    results.push(DiffResult {
                        diff_type: DiffType::RemovedSeats,
                        encounter_diff_only: enc,
                    });
                }
            }
        }
    }

    results
}
