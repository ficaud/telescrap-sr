mod proxy_core;
mod proxy_api;

pub use proxy_core::ProxyManager;
pub use proxy_api::{PROXY_MANAGER, PROXY_ENABLED, set_proxy_enabled, retry_with_proxy};
