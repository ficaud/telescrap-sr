use curl::easy::Easy;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct Proxy {
    url: String,
}

/// Manages a pool of proxy URLs loaded from a file or fetched from the
/// ProxyScrape API, with round-robin rotation and automatic failover.
///
/// Priority order:
/// 1. If `PROXIES_FILE` env var is set → load proxies from that file
/// 2. Otherwise → fetch from the ProxyScrape API
///
/// Each call to `get()` returns the next proxy in the rotation. When a
/// proxy fails, it is removed from the available pool. Use `reset_all()`
/// to re-fetch a fresh list from the API and restart the rotation.
pub struct ProxyManager {
    all: Mutex<Vec<Proxy>>,
    available: Mutex<Vec<Proxy>>,
    cursor: Mutex<usize>,
}

impl ProxyManager {
    /// Creates a new `ProxyManager` with empty proxy lists and a cursor set to 0.
    /// The proxy list will be lazily loaded on first use.
    ///
    /// # Return
    /// A new instance of `ProxyManager` ready to manage proxy URLs.
    fn new() -> Self {
        ProxyManager {
            all: Mutex::new(vec![]),
            available: Mutex::new(vec![]),
            cursor: Mutex::new(0),
        }
    }

    /// Fetches the proxy list from a file (`PROXIES_FILE` env) or from
    /// the ProxyScrape API (called once on first use).
    fn ensure_fetched(&self) {
        let mut all = self.all.lock().unwrap();
        if !all.is_empty() {
            return;
        }

        // Priority 1: load from file if PROXIES_FILE is set
        if let Ok(path) = std::env::var("PROXIES_FILE") {
            match Self::fetch_proxies_from_file(&path) {
                Ok(proxies) if !proxies.is_empty() => {
                    let count = proxies.len();
                    *all = proxies.clone();
                    let start = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as usize
                        % count;
                    *self.cursor.lock().unwrap() = start;
                    *self.available.lock().unwrap() = proxies;
                    println!("[PROXY] {} proxy(ies) loaded from {}", count, path);
                    return;
                }
                Ok(_) => eprintln!("[PROXY] File {} is empty", path),
                Err(e) => eprintln!("[PROXY] Failed to read {}: {}", path, e),
            }
        }

        // Priority 2: fetch from ProxyScrape API
        match Self::fetch_proxies_from_api() {
            Ok(proxies) => {
                let count = proxies.len();
                *all = proxies.clone();
                let start = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize
                    % count;
                *self.cursor.lock().unwrap() = start;
                *self.available.lock().unwrap() = proxies;
                println!("[PROXY] {} proxy(ies) loaded from ProxyScrape API", count);
            }
            Err(e) => {
                eprintln!("[PROXY] Failed to fetch proxy list: {e}");
            }
        }
    }

    /// Performs a synchronous GET request to the ProxyScrape API and
    /// parses the response into a list of proxy URLs.
    ///
    /// # Arguments
    /// * `url` - The URL of the ProxyScrape API endpoint to fetch proxies
    ///
    /// # Return
    /// A `Result` containing a vector of `Proxy` instances on success, or an error if the request or parsing fails.
    fn fetch_proxies_from_api() -> Result<Vec<Proxy>, Box<dyn std::error::Error>> {
        let url = "https://api.proxyscrape.com/v4/free-proxy-list/get?request=display_proxies&proxy_format=protocolipport&format=text&protocol=http&country=fr";

        let mut easy = Easy::new();
        easy.url(url)?;
        easy.follow_location(true)?;
        // Identify ourselves to avoid being blocked
        let mut headers = curl::easy::List::new();
        headers.append("User-Agent: Telescrap/1.0")?;
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

        let text = String::from_utf8(response)?;
        let proxies: Vec<Proxy> = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| Proxy {
                url: line.to_string(),
            })
            .collect();

        if proxies.is_empty() {
            eprintln!("[PROXY] ProxyScrape API returned an empty list");
        }

        Ok(proxies)
    }

    /// Reads proxies from a file. Supports two formats per line:
    /// - Full URL: `http://user:pass@ip:port`
    /// - Short form: `ip:port` or `ip:port:user:pass` (automatically
    ///   prefixed with `http://`)
    ///
    /// # Arguments
    /// * `path` - The file path to read proxy URLs from
    ///
    /// # Return
    /// A `Result` containing a vector of `Proxy` instances on success, or an error if the file cannot be read or contains malformed lines.
    fn fetch_proxies_from_file(path: &str) -> Result<Vec<Proxy>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let proxies: Vec<Proxy> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| Proxy {
                url: Self::normalize_proxy_url(line),
            })
            .collect();
        Ok(proxies)
    }

    /// Converts a single proxy line to a full URL.
    ///
    /// Supports:
    /// - `http://user:pass@1.2.3.4:8080` → passed through
    /// - `1.2.3.4:8080` → `http://1.2.3.4:8080`
    /// - `1.2.3.4:8080:user:pass` → `http://user:pass@1.2.3.4:8080`
    fn normalize_proxy_url(line: &str) -> String {
        if line.contains("://") {
            return line.to_string();
        }

        let parts: Vec<&str> = line.split(':').collect();
        match parts.len() {
            // ip:port
            2 => format!("http://{}:{}", parts[0], parts[1]),
            // ip:port:user:pass
            4 => format!("http://{}:{}@{}:{}", parts[2], parts[3], parts[0], parts[1]),
            _ => {
                eprintln!("[PROXY] Skipping malformed proxy line: {}", line);
                String::new()
            }
        }
    }

    /// Returns the next proxy URL in the rotation, or `None` if the
    /// pool is empty.
    ///
    /// # Return
    /// An `Option<String>` containing the next proxy URL if available, or `None` if the proxy pool is empty.
    pub fn get(&self) -> Option<String> {
        self.ensure_fetched();
        let available = self.available.lock().unwrap();
        if available.is_empty() {
            return None;
        }
        let mut cursor = self.cursor.lock().unwrap();
        // Ensure cursor is within bounds
        if *cursor >= available.len() {
            *cursor = 0;
        }
        let idx = *cursor;
        *cursor = (*cursor + 1) % available.len();
        Some(available[idx].url.clone())
    }

    /// Marks a proxy URL as failed, removing it from the available pool.
    ///
    /// # Arguments
    /// * `url` - The proxy URL to mark as failed and remove from the available pool
    pub fn mark_failed(&self, url: &str) {
        self.ensure_fetched();
        let mut available = self.available.lock().unwrap();
        let before = available.len();
        available.retain(|p| p.url != url);
        let removed = before - available.len();
        if removed > 0 {
            // Adjust cursor so we don't skip a proxy
            let mut cursor = self.cursor.lock().unwrap();
            if *cursor > 0 {
                *cursor = cursor.saturating_sub(1);
            }
            println!(
                "[PROXY] Removed {} — {} proxy(ies) remaining",
                url,
                available.len()
            );
        }
    }

    /// Re-fetches the proxy list from the API and restarts the
    /// rotation. Call this when all proxies have been exhausted.
    pub fn reset_all(&self) {
        let mut all = self.all.lock().unwrap();
        all.clear();
        self.ensure_fetched();
    }

    /// Number of currently available (non-failed) proxies.
    ///
    /// # Return
    /// The count of currently available proxies in the pool.
    pub fn available_count(&self) -> usize {
        self.ensure_fetched();
        self.available.lock().unwrap().len()
    }

    /// Total number of proxies fetched (including failed ones).
    ///
    /// # Return
    /// The total count of proxies that have been fetched and are being managed,
    /// including those that have been marked as failed.
    pub fn total_count(&self) -> usize {
        self.ensure_fetched();
        self.all.lock().unwrap().len()
    }
}

/// Global proxy manager, initialised lazily on first use.  The proxy
/// list is fetched from the ProxyScrape API when the first request is
/// made.
pub static PROXY_MANAGER: LazyLock<ProxyManager> = LazyLock::new(ProxyManager::new);

/// Executes `f` with automatic proxy selection and failover.
///
/// - If the proxy pool is empty (API unavailable), `f(None)` is called
///   once (no proxy).
/// - Otherwise the next proxy in the rotation is chosen and passed as
///   `Some(url)`. If `f` returns an error, that proxy is marked as
///   failed and the next available proxy is tried, repeating until one
///   succeeds or all proxies are exhausted.
///
/// # Example
/// ```ignore
/// let html = retry_with_proxy(|proxy| {
///     let mut easy = Easy::new();
///     if let Some(p) = proxy {
///         easy.proxy(p)?;
///     }
///     // ... configure easy ...
///     Ok(result)
/// })?;
/// ```
pub fn retry_with_proxy<F, T>(mut f: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnMut(Option<&str>) -> Result<T, Box<dyn std::error::Error>>,
{
    // Attempt to fetch proxies. If the pool is empty (API unavailable),
    // call the closure once without a proxy.
    if PROXY_MANAGER.total_count() == 0 {
        return f(None);
    }

    let mut last_err = None;
    while let Some(proxy_url) = PROXY_MANAGER.get() {
        // Extract and display ip:port from the proxy URL (e.g. "http://1.2.3.4:8080")
        let display = proxy_url.split("://").nth(1).unwrap_or(&proxy_url);
        println!("[PROXY] Using proxy: {}", display);

        match f(Some(&proxy_url)) {
            Ok(val) => return Ok(val),
            Err(e) => {
                PROXY_MANAGER.mark_failed(&proxy_url);
                last_err = Some(e);
                if PROXY_MANAGER.available_count() == 0 {
                    break;
                }
            }
        }
    }
    Err(last_err
        .unwrap_or_else(|| "All proxies exhausted and no direct connection configured".into()))
}
