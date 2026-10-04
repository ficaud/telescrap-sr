/// This module defines the data structures related to seats in the context of rugby matches, including
/// seat composition, seat information, and actions that can be performed on a seat (such as adding it to a shopping cart).
///

/// SeatComposition represents the details of a seat's location and category, including the category, access type, row, and seat number.
#[derive(Debug, Clone)]
pub struct SeatComposition {
    pub category: String,
    pub bloc: String,
    pub row: String,
    pub seat_number: u64,
}

/// SeatInfo contains the full name of the seat and its composition details.
#[derive(Debug, Clone)]
pub struct SeatInfo {
    pub full_name: String,
    pub composition: SeatComposition,
    pub preview_url: Option<String>,
}

/// SeatAction represents the actions that can be performed on a seat, such as adding it to a shopping cart.
#[derive(Debug, Clone)]
pub struct SeatAction {
    pub pack_id: String,
    pub ticket_id: String,
    pub ajax_url: String,
    pub price_min: String,
    pub price_max: String,
    pub libraries: String,
    pub form_build_id: String,
    pub form_token: String,
}

/// Seat represents a seat available for a rugby match, including its information and the actions that can be performed on it (like adding to cart).
#[derive(Debug, Clone)]
pub struct Seat {
    pub seat_info: SeatInfo,
    pub price: String,
    pub actions: SeatAction,
}
