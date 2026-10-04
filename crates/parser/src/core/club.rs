/// This module defines the Club struct and the ClubType enum, which represent rugby clubs and their types.

/// ClubType isn an enumeration of the different rugby clubs that can be parsed by the application.
/// It currently includes
///
/// StadeRochelais
/// UnionBordeauxBegles -> not implemented yet
#[derive(Debug, Clone)]
pub enum ClubType {
    StadeRochelais,
    UnionBordeauxBegles,
}

/// Club structure that represents a rugby club, including its name, type, and the URL of its ticketing website.
#[derive(Debug, Clone)]
pub struct Club {
    pub name: String,
    pub club_type: ClubType,
    pub url: String,
}

/// Implementation of the Club struct, including a constructor and a method to get the URL of the club's ticketing website.
impl Club {
    /// Creates a new Club instance with the given name, type, and URL.
    ///
    /// # Arguments
    /// * `name` - The name of the club
    /// * `club_type` - The type of the club (from the ClubType enum
    /// * `url` - The URL of the club's ticketing website
    /// # Returns
    /// A new instance of the Club struct with the provided information
    pub fn new(name: String, club_type: ClubType, url: String) -> Self {
        Self {
            name,
            club_type,
            url,
        }
    }

    /// Returns the URL of the club's ticketing website.
    ///
    /// # Returns
    /// A string slice containing the URL of the club's ticketing website
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Utility function to get the ClubType enum variant from a club name string.
    ///
    /// # Arguments
    /// * `name` - The name of the club as a string
    /// # Returns
    /// The corresponding ClubType enum variant for the given club name
    pub fn get_type_from_name(name: &str) -> ClubType {
        match name.to_lowercase().as_str() {
            "staderochelais" => ClubType::StadeRochelais,
            "unionbordeauxbegles" => ClubType::UnionBordeauxBegles,
            _ => panic!("Unknown club name: {}", name),
        }
    }
}
