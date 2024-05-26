use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use serde::Deserialize;

pub mod parse;
pub mod routes;

pub const APP_DEFAULT_PORT: u16 = 3000;
pub static APP_NAME: OnceLock<String> = OnceLock::new();
pub static APP_VERSION: OnceLock<String> = OnceLock::new();
#[cfg(feature = "proxy")]
pub static APP_PORT: OnceLock<u16> = OnceLock::new();

// todo: add configuration options
#[derive(Clone, Deserialize)]
pub struct FurssOptions {
    _flaresolverr: Option<String>,
    _proxy: Option<String>,
    _proxy_username: Option<String>,
    _proxy_password: Option<String>,
    _disable_cache: Option<bool>,
    full: Option<bool>,
    _number_items: Option<u16>,
}

#[cfg(feature = "proxy")]
#[derive(Deserialize, Clone)]
pub struct Key {
    pub url: String,
    pub time: String,
}

#[cfg(feature = "proxy")]
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Mutex<HashMap<String, HashMap<String, Key>>>>,
}
