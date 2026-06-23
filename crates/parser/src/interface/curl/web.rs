use super::proxy::retry_with_proxy;
use crate::controller::html_extract::FetchHtml;
use crate::core::seat::SeatAction;
use curl::easy::Easy;
use curl::easy::List;
use std::time::Duration;

/// WebClient is a struct that implements the FetchHtml trait, allowing it to fetch HTML content from a URL
pub struct WebClient {
    pub cookie_jar: Option<String>,
}

/// Webclient implementation functions for creating new instances with or without session cookies
impl WebClient {
    pub fn new() -> Self {
        Self { cookie_jar: None }
    }

    pub fn with_session(cookie_jar: &str) -> Self {
        Self { cookie_jar: Some(cookie_jar.to_string()) }
    }
}

/// Implements the FetchHtml trait for WebClient, allowing it to fetch HTML content,
/// add a seat to the cart, -> not implemented yet
/// and connect to the shop with credentials to add a seat to the cart. -> not implemented yet
impl FetchHtml for WebClient {
    fn get_html(&self, url: &str) -> Result<String, String> {
        fetch_html(url, self.cookie_jar.as_deref()).map_err(|e| e.to_string())
    }

    fn add_to_cart(&self, action: &SeatAction) -> Result<(), String> {
        add_to_cart(action, self.cookie_jar.as_deref()).map_err(|e| e.to_string())
    }

    fn connect_and_add_to_cart(&self, email: &str, password: &str, action: &SeatAction) -> Result<(), String> {
        connect_and_add_to_cart(email, password, action).map_err(|e| e.to_string())
    }
}

/// Fetches HTML content from the given URL, optionally using a cookie jar for session management.
///
/// # Arguments
/// * `url` - The URL from which to fetch HTML content
/// * `cookie_jar` - An optional path to a cookie jar file for managing session cookies
/// # Returns
/// A Result containing the fetched HTML content as a String, or an error if the fetch operation fails
fn fetch_html(url: &str, cookie_jar: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    retry_with_proxy(|proxy| {
        let mut easy = Easy::new();
        easy.url(url)?;
        if let Some(jar) = cookie_jar {
            easy.cookie_file(jar)?;
            easy.cookie_jar(jar)?;
        }
        if let Some(p) = proxy {
            easy.proxy(p)?;
        }
        easy.connect_timeout(Duration::from_secs(5))?;
        easy.timeout(Duration::from_secs(15))?;
        easy.follow_location(true)?;
        let mut html = Vec::new();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                html.extend_from_slice(data);
                Ok(data.len())
            })?;
            transfer.perform()?;
        }
        Ok(String::from_utf8(html)?)
    })
}

pub fn connect_to_shop(
    email: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("[WEB] Starting shop login for {}", email);
    let cookie_jar = format!(
        "{}/sr_session_{}_{}.jar",
        std::env::temp_dir().display(),
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    );
    let _ = std::fs::remove_file(&cookie_jar);
    println!("[WEB] Cookie jar path: {}", cookie_jar);

    let login_url = "https://billetterie.staderochelais.com/fr/user/login";
    println!("[WEB] Fetching login page: {}", login_url);
    let login_page = fetch_with_jar(login_url, &cookie_jar)?;
    let form_build_id = extract_form_build_id(&login_page)
        .ok_or_else(|| std::io::Error::other("form_build_id introuvable dans la page de login"))?;
    println!("[WEB] Login page form_build_id extracted: {}", form_build_id);

    println!("[WEB] Posting login email step for {}", email);
    let step2_html = post_email_step(email, &form_build_id, &cookie_jar)?;
    if step2_html.contains("/register") {
        eprintln!("[WEB] Login email step redirected to register for {}", email);
        return Err(std::io::Error::other("Compte introuvable pour cet email").into());
    }

    let form_build_id2 = extract_form_build_id(&step2_html)
        .ok_or_else(|| std::io::Error::other("form_build_id introuvable après étape email"))?;
    println!("[WEB] Password step form_build_id extracted: {}", form_build_id2);

    println!("[WEB] Posting password step for {}", email);
    let _ = post_password_step(email, password, &form_build_id2, login_url, &cookie_jar)?;

    println!("[WEB] Verifying authenticated basket session");
    let basket = fetch_with_jar("https://billetterie.staderochelais.com/fr/basket", &cookie_jar)?;
    if basket.contains("user-logged-in") || basket.contains("Se déconnecter") {
        println!("[WEB] Shop login succeeded for {}", email);
        Ok(cookie_jar)
    } else {
        eprintln!("[WEB] Shop login failed for {}: basket page does not look authenticated", email);
        Err(std::io::Error::other("Échec d'authentification (session non connectée)").into())
    }
}

#[allow(dead_code)]
fn fetch_with_jar(url: &str, cookie_jar: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.cookie_file(cookie_jar)?;
    easy.cookie_jar(cookie_jar)?;
    easy.follow_location(true)?;

    let mut html = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            html.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(String::from_utf8(html)?)
}

pub fn extract_form_token(html: &str) -> Option<String> {
    if let Some(pos) = html.find("name=\"form_token\"") {
        let after = &html[pos..];
        if let Some(val_pos) = after.find("value=\"") {
            let val_start = val_pos + 7;
            if let Some(val_end) = after[val_start..].find('"') {
                return Some(after[val_start..val_start + val_end].to_string());
            }
        }
    }

    if let Some(pos) = html.find("form_token\\u0022 value=\\u0022") {
        let start = pos + "form_token\\u0022 value=\\u0022".len();
        if let Some(end) = html[start..].find("\\u0022") {
            return Some(html[start..start + end].to_string());
        }
    }

    None
}

pub fn extract_form_build_id(html: &str) -> Option<String> {
    if let Some(pos) = html.find("name=\"form_build_id\"") {
        let after = &html[pos..];
        if let Some(val_pos) = after.find("value=\"") {
            let val_start = val_pos + 7;
            if let Some(val_end) = after[val_start..].find('"') {
                return Some(after[val_start..val_start + val_end].to_string());
            }
        }
    }

    if let Some(pos) = html.find("form_build_id\\u0022 value=\\u0022") {
        let start = pos + "form_build_id\\u0022 value=\\u0022".len();
        if let Some(end) = html[start..].find("\\u0022") {
            return Some(html[start..start + end].to_string());
        }
    }

    None
}

#[allow(dead_code)]
fn post_email_step(
    email: &str,
    form_build_id: &str,
    cookie_jar: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let login_url = "https://billetterie.staderochelais.com/fr/user/login";
    let body = format!(
        "name={}&form_build_id={}&form_id=user_login_form&op=Continuer",
        urlencode(email),
        urlencode(form_build_id),
    );
    println!("[WEB] POST email step with cookie jar {}", cookie_jar);

    let mut easy = Easy::new();
    easy.url(login_url)?;
    easy.post(true)?;
    easy.post_fields_copy(body.as_bytes())?;
    easy.cookie_file(cookie_jar)?;
    easy.cookie_jar(cookie_jar)?;
    easy.follow_location(true)?;

    let mut headers = List::new();
    headers.append("Content-Type: application/x-www-form-urlencoded")?;
    headers.append(&format!("Referer: {}", login_url))?;
    easy.http_headers(headers)?;

    let mut response = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(String::from_utf8(response)?)
}

#[allow(dead_code)]
fn post_password_step(
    email: &str,
    password: &str,
    form_build_id: &str,
    url: &str,
    cookie_jar: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let body = format!(
        "name={}&pass={}&form_build_id={}&form_id=user_login_form&op=Se+connecter",
        urlencode(email),
        urlencode(password),
        urlencode(form_build_id),
    );
    println!("[WEB] POST password step with cookie jar {}", cookie_jar);

    let mut easy = Easy::new();
    easy.url(url)?;
    easy.post(true)?;
    easy.post_fields_copy(body.as_bytes())?;
    easy.cookie_file(cookie_jar)?;
    easy.cookie_jar(cookie_jar)?;
    easy.follow_location(true)?;

    let mut headers = List::new();
    headers.append("Content-Type: application/x-www-form-urlencoded")?;
    headers.append(&format!("Referer: {}", url))?;
    easy.http_headers(headers)?;

    let mut response = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(String::from_utf8(response)?)
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn connect_and_add_to_cart(
    email: &str,
    password: &str,
    action: &SeatAction,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "[WEB] Starting authenticated add-to-cart flow for {} (pack {} ticket {})",
        email,
        action.pack_id,
        action.ticket_id
    );
    let cookie_jar = connect_to_shop(email, password)?;

    let base_url = "https://billetterie.staderochelais.com";
    let resale_page_url = format!("{}{}", base_url, action.ajax_url.split('?').next().unwrap_or(""));
    println!("[WEB] Refreshing resale page before add-to-cart: {}", resale_page_url);
    let fresh_html = fetch_with_jar(&resale_page_url, &cookie_jar)?;

    let fresh_action = SeatAction {
        form_build_id: extract_form_build_id(&fresh_html).unwrap_or_else(|| action.form_build_id.clone()),
        form_token: extract_form_token(&fresh_html).unwrap_or_else(|| action.form_token.clone()),
        ..action.clone()
    };
    println!(
        "[WEB] Refreshed action tokens for pack {} ticket {}",
        fresh_action.pack_id,
        fresh_action.ticket_id
    );

    let res = add_to_cart(&fresh_action, Some(&cookie_jar));

    let _ = std::fs::remove_file(&cookie_jar);
    println!("[WEB] Session cookie jar removed: {}", cookie_jar);

    res
}


pub fn add_to_cart(
    action: &SeatAction,
    cookie_jar: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "https://billetterie.staderochelais.com";
    let url = format!("{}{}&_wrapper_format=drupal_ajax", base_url, action.ajax_url);
    println!(
        "[WEB] POST add-to-cart for pack {} ticket {}{}",
        action.pack_id,
        action.ticket_id,
        if cookie_jar.is_some() { " with session" } else { " without session" }
    );

    let body = format!(
        "nb_tickets_selector=0\
        &price_slider={min}%2C{max}\
        &sort=price-asc\
        &selected_tickets_{pack_id}={ticket_id}\
        &tickets_selected=\
        &resale_pack_selected={pack_id}\
        &form_build_id={form_build_id}\
        &form_token={form_token}\
        &form_id=hubber_resale_add_to_cart_form\
        &_triggering_element_name=add_pack\
        &_triggering_element_value=Ajouter+%C3%A0+mon+panier\
        &_drupal_ajax=1\
        &ajax_page_state%5Btheme%5D=hubber_reference8\
        &ajax_page_state%5Btheme_token%5D=\
        &ajax_page_state%5Blibraries%5D={libraries}",
        min = urlencode(&action.price_min),
        max = urlencode(&action.price_max),
        pack_id = urlencode(&action.pack_id),
        ticket_id = urlencode(&action.ticket_id),
        form_build_id = urlencode(&action.form_build_id),
        form_token = urlencode(&action.form_token),
        libraries = urlencode(&action.libraries),
    );

    let mut easy = Easy::new();
    easy.url(&url)?;
    easy.post(true)?;
    easy.post_fields_copy(body.as_bytes())?;
    if let Some(jar) = cookie_jar {
        easy.cookie_file(jar)?;
        easy.cookie_jar(jar)?;
    }

    let mut headers = List::new();
    headers.append("Accept: application/json, text/javascript, */*; q=0.01")?;
    headers.append("Content-Type: application/x-www-form-urlencoded; charset=UTF-8")?;
    headers.append("X-Requested-With: XMLHttpRequest")?;
    headers.append(&format!("Referer: {}{}", base_url, action.ajax_url.split('?').next().unwrap_or("")))?;
    easy.http_headers(headers)?;

    let mut response = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let json_str = String::from_utf8(response)?;
    println!("[WEB] Add-to-cart response size: {} bytes", json_str.len());
    if json_str.contains("p\u{e9}rim\u{e9}")
        || json_str.contains("p%C3%A9rim%C3%A9")
        || (json_str.contains("alert-danger") && json_str.contains("formulaire"))
    {
        eprintln!("[WEB] Add-to-cart rejected: expired form or invalid session");
        return Err(std::io::Error::other("Formulaire périmé ou session invalide").into());
    }

    println!("[WEB] Add-to-cart request completed successfully");

    Ok(())
}

