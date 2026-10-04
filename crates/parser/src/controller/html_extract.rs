/// This module defines the FetchHtml trait, which abstracts the functionality for fetching HTML content from a URL,
/// as well as actions related to adding seats to a shopping cart. It also includes a utility function to extract HTML content using a provided implementation of the FetchHtml trait.
use crate::core::seat::SeatAction;

/// FetchHtml is a trait that defines the methods for fetching HTML content from a URL, adding a seat to a shopping cart, and connecting to the shop with credentials to add a seat to the cart.
pub trait FetchHtml {
    fn get_html(&self, url: &str) -> Result<String, String>;
    fn add_to_cart(&self, action: &SeatAction) -> Result<(), String>;
    fn connect_and_add_to_cart(
        &self,
        email: &str,
        password: &str,
        action: &SeatAction,
    ) -> Result<(), String>;
}
