/// This module defines the club manager, which provides a function to create a new Club instance based on a given ClubType.
/// It uses the Club struct and ClubType enum from the core::club module to create and return a new Club instance with the appropriate name, type, and ticketing URL.
///
/// Any new inclusion of clubs should be added to the ClubType enum and the new function should be updated to handle the new club type accordingly.
use crate::core::club::{Club, ClubType};

/// Creates a new Club instance based on the provided ClubType.
///
/// # Arguments
/// * `club_type` - The type of the club for which to create a new Club instance, as defined in the ClubType enum
/// # Returns
/// A new instance of the Club struct with the appropriate name, type, and ticketing URL based on the provided ClubType
pub fn new(club_type: ClubType) -> Club {
    match club_type {
        ClubType::StadeRochelais => Club::new(
            "Stade Rochelais".to_string(),
            ClubType::StadeRochelais,
            "https://billetterie.staderochelais.com/fr".to_string(),
        ),
        ClubType::UnionBordeauxBegles => Club::new(
            "Union Bordeaux Bègles".to_string(),
            ClubType::UnionBordeauxBegles,
            "https://billetterie.ubbrugby.com/fr".to_string(),
        ),
    }
}
