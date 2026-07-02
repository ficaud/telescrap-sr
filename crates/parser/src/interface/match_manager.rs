/// This module manages the retrieval of match and seat information for rugby clubs, as well as interactions with the shopping cart.
///
/// It provides the main parser functions that should be used in the rest of the application to get matches and seats informations
use crate::{controller::encounter_store::StoreEncounters, core::{
    club::{
        Club,
        ClubType
    }, encounter::{
        Encounter,
        MatchNature,
    }, seat::{Seat, SeatComposition}
}};
use crate::controller::html_extract::FetchHtml;
use crate::app::clubs::{
    parsers::{
        ParseSeat,
        ParseMatch,
        ParseSeatPreview,
    },
    larochelle::{
        parse_match::LarochellMatchParser,
        parse_seat::LarochellSeatParser,
        parse_seat_preview::LarochellSeatPreviewParser,
    }
};
use crate::interface::curl::web::{WebClient, connect_and_add_to_cart, ProxyMode};
use crate::interface::storage::EncounterStore;

fn matchs_db_path() -> String {
    std::env::var("MATCHS_DB_PATH").unwrap_or_else(|_| "matchs.db".to_string())
}

/// Connects to the shop with the given seat information and adds it to the cart.
pub fn connect_and_add_seat_to_cart(email: String, password: String, seat: Seat) -> Result<(), Box<dyn std::error::Error>> {
    connect_and_add_to_cart(&email, &password, &seat.actions)
}

/// Prints all encounter records stored in the database.
pub fn print_db_contents() {
    let db = match EncounterStore::open(matchs_db_path()) {
        Ok(db) => db,
        Err(e) => { eprintln!("Failed to open database: {}", e); return; }
    };
    match db.get_all() {
        Ok(records) if records.is_empty() => println!("Database is empty."),
        Ok(records) => {
            println!("Database contains {} record(s):", records.len());
            for r in records {
                println!(
                    "  [{active}] {title} | {date} | club: {club} | link: {link}",
                    active = if r.resale_active { "active" } else { "inactive" },
                    title = r.title,
                    date = r.date,
                    club = r.club_type,
                    link = r.resale_link,
                );
            }
        }
        Err(e) => eprintln!("Storage error: {}", e),
    }
}

/// Fetches encounters with seats for a given club and match nature
///
/// # Arguments
/// * `club` - The club to fetch matches for
/// * `match_type` - The nature of the match to fetch
/// # Returns
/// A list of encounters with their seats information populated
///
pub fn get_seats_from_matches(club: Club, match_type: MatchNature) -> Vec<Encounter> {
    let client = WebClient::new(ProxyMode::Disabled);
    let db = EncounterStore::open(matchs_db_path()).unwrap();

    // Priority 1: try cached active resale links from DB first
    if let Ok(records) = db.get_active_resale_links() {
        let mut cached_results: Vec<Encounter> = Vec::new();

        for record in &records {

            let encounter = Encounter::new(
                Club::get_type_from_name(&record.club_type),
                record.title.clone(),
                record.date.clone(),
                match_type,
                Some(record.resale_link.clone()),
            );

            let mut with_seats = get_encounters_with_seats(vec![encounter], &client);
            if let Some(e) = with_seats.first_mut() {
                let has_seats = e.seats.as_ref().map_or(false, |s| !s.is_empty());
                if has_seats {
                    // Cache hit — keep it active and collect
                    eprintln!(
                        "[CACHE] Reusing cached resale link for '{}' ({}): {} seats found",
                        record.title, record.date,
                        e.seats.as_ref().map_or(0, |s| s.len()),
                    );
                    cached_results.extend(with_seats);
                } else {
                    // Only mark the link as inactive if the match date has passed.
                    // If the match is still in the future, keep it active so we retry.
                    if e.date_passed() {
                        eprintln!(
                            "[CACHE] Match '{}' ({}) has passed, disabling cached link",
                            record.title, record.date,
                        );
                        let stale = Encounter::new(
                            Club::get_type_from_name(&record.club_type),
                            record.title.clone(),
                            record.date.clone(),
                            match_type,
                            None,
                        );
                        if let Err(e) = db.upsert(&stale) {
                            eprintln!("Storage error while marking stale: {}", e);
                        }
                    } else {
                        eprintln!(
                            "[CACHE] No seats for '{}' ({}), but match not passed yet — keeping link active",
                            record.title, record.date,
                        );
                    }
                }
            }
        }

        if !cached_results.is_empty() {
            return cached_results;
        }
    }

    println!("[CACHE] No active resale links found in DB, falling back to web scraping for matches.");
    // Priority 2: fallback — parse from web (original behavior)
    let matches = get_matches_from_type_and_club(match_type, club);

    let matches: Vec<Encounter> = matches.into_iter().map(|mut encounter| {
        // If the page didn't return a resale link, check DB for an existing active one
        if encounter.resale_link.is_none() {
            if let Ok(Some(record)) = db.get_by_stable_id(&encounter.title, &encounter.date) {
                println!("[DB] Found existing record for '{}' ({}), using cached resale link.", record.title, record.date);
                if record.resale_active {
                    encounter.resale_link = Some(record.resale_link);
                }
            }
        }
        // Upsert the (possibly enriched) encounter
        if let Err(e) = db.upsert(&encounter) {
            eprintln!("Storage error for '{}': {}", encounter.title, e);
        }
        encounter
    }).collect();

    get_encounters_with_seats(matches, &client)
}

/// Fectes match's seats from a given match title, club and match nature.
/// It first tries to find an active resale link in the database for the given title, if it finds one it fetches seats from it,
/// otherwise it falls back to fetching matches from the web and filtering by title.
///
/// # Arguments
/// * `match_title` - The title of the match to fetch seats for
/// * `club` - The club to fetch matches for
/// * `match_type` - The nature of the match to fetch
/// # Returns
/// A list of encounters with their seats information populated (which is 1 if a match with the given title is found, 0 otherwise)
///
pub fn get_seats_from_match_title(match_title: String, club: Club, match_type: MatchNature) -> Vec<Encounter> {
    // We need rotating proxy here because we are directly fecthing the resale link save in db
    let client = WebClient::new(ProxyMode::Rotating);
    let db = EncounterStore::open(matchs_db_path()).unwrap();

    // Get all occurence from data base
    match db.get_all() {
        // If there is something
        Ok(records) => {
            // try to find a record with the same title and an active resale link
            for record in records {
                if record.title == match_title && record.resale_active {
                        // If that records has an active resale link, try to fetch seats from it
                        let link = &record.resale_link;
                        match client.get_html(link) {
                            Ok(_) => {
                                let enc = Encounter::new(
                                    Club::get_type_from_name(&record.club_type),
                                    record.title,
                                    record.date,
                                    match_type,
                                    Some(record.resale_link));

                                // Stop here, we return the seats from this match (vector of 1 encounter)
                                return get_encounters_with_seats(vec![enc], &client);
                            }
                            Err(e) => eprintln!("Error fetching {}: {}", link, e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Storage error while retrieving matches: {}", e),
    }

    // If there is no resale link, get matches
    let matches = get_matches_from_type_and_club(match_type, club);
    // Filter by the one with the right title
    let filtered = matches.into_iter().filter(|e| e.title == match_title).collect();
    // Get seats from the filtered match (vector of 0 or 1 encounter)
    get_encounters_with_seats(filtered, &client)
}

/// Internal function to fetch seats for a list of encounters, given a client to fetch HTML content.
///
/// # Arguments
/// * `matches` - The list of encounters to retrieve seats for
/// * `client` - The client to use for fetching HTML content
/// # Returns
/// A list of encounters with their seats information populated
fn get_encounters_with_seats(matches: Vec<Encounter>, client: &impl FetchHtml) -> Vec<Encounter> {
    matches.into_iter().map(|mut encounter| {
        if let Some(link) = encounter.resale_link.clone() {
            match client.get_html(&link) {
                Ok(html) => encounter.set_seats(get_seats(&html, encounter.clone())),
                Err(e) => eprintln!("Error fetching {}: {}", link, e),
            }
        } else {
            encounter.set_seats(Vec::new());
        }
        encounter
    }).collect()
}

/// Internal function to fetch matches for a given club and client, optionally filtered by match nature.
///
/// # Arguments
/// * `club` - The club to fetch matches for
/// * `client` - The client to use for fetching HTML content
/// * `match_type` - Optional filter to return only matches of a specific nature
/// # Returns
/// A list of encounters matching the specified criteria
///
fn get_matches(club: &Club, client: &impl FetchHtml, match_type: MatchNature) -> Vec<Encounter> {

    // Step 0: Set the correct parser from the club
    let parser: &dyn ParseMatch = match club.club_type {
        ClubType::StadeRochelais => &LarochellMatchParser,
        ClubType::UnionBordeauxBegles => todo!("Bordeaux parser not yet implemented"),
    };

    // Step 1 : Extract HTML content from the club's URL
    let content = client.get_html(club.get_url());

    // Step 2 : Parse the HTML content to find matches (encounters) information
    let matches = parser.parse_match(&content.unwrap_or_default());

    // Step 3 : filter by nature you want to get
    matches.into_iter().filter(|encounter| encounter.nature == match_type).collect()
}

/// Internal function to fetch seats for a given encounter, based on its club type and resale link.
/// It selects the correct seat parser based on the club type and uses it to parse the seats information from the HTML content of the resale link.
///
/// # Arguments
/// * `html` - The HTML content to parse for seat information
/// * `encounter` - The encounter for which to fetch seats information (used to determine the correct parser based on club type)
/// # Returns
/// A list of seats associated with the given encounter, parsed from the HTML content
fn get_seats(html: &str, encounter: Encounter) -> Vec<Seat> {
    let parser: &dyn ParseSeat = match encounter.club_type {
        ClubType::StadeRochelais => &LarochellSeatParser,
        ClubType::UnionBordeauxBegles => todo!("Bordeaux parser not yet implemented"),
    };

    parser.parse_seat(html, encounter)
}

/// Internal function to fetch all match encounters for a given club and match nature
///
/// # Arguments
/// * `match_type` - Optional filter to return only matches of a specific nature
/// * `club` - The club to fetch matches for
/// # Returns
/// A list of encounters matching the specified criteria
fn get_matches_from_type_and_club(match_type: MatchNature, club: Club) -> Vec<Encounter> {
    let client = WebClient::new(ProxyMode::Rotating);
    get_matches(&club, &client, match_type)
}

/// Fetches the Pacifa3d preview image URL for a given seat composition, dispatching to the correct
/// parser based on the club type.
///
/// # Arguments
/// * `club` - The club the seat belongs to (used to select the correct parser)
/// * `composition` - The seat composition (access, row, seat number) to resolve
/// # Returns
/// `Some(url)` if the preview image was found, `None` otherwise
pub fn get_seat_preview(club: &Club, composition: &SeatComposition) -> Option<String> {
    let parser: &dyn ParseSeatPreview = match club.club_type {
        ClubType::StadeRochelais => &LarochellSeatPreviewParser,
        ClubType::UnionBordeauxBegles => todo!("Bordeaux preview parser not yet implemented"),
    };
    parser.fetch_preview_url(composition)
}
