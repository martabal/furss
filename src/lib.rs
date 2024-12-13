use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serde::Deserialize;
use tokio::sync::Mutex;

pub mod parse;
pub mod routes;

pub const APP_DEFAULT_PORT: u16 = 3000;
pub static APP_NAME: OnceLock<String> = OnceLock::new();
pub static APP_VERSION: OnceLock<String> = OnceLock::new();
#[cfg(feature = "proxy")]
pub static APP_PORT: OnceLock<u16> = OnceLock::new();

#[derive(Clone, Deserialize)]
pub struct FurssOptions {
    flaresolverr: Option<String>,
    _proxy: Option<String>,
    _proxy_username: Option<String>,
    _proxy_password: Option<String>,
    #[cfg(feature = "proxy")]
    _disable_cache: Option<bool>,
    full: Option<bool>,
    number_items: Option<u16>,
}

#[cfg(feature = "proxy")]
type Cache = Arc<Mutex<HashMap<String, Arc<Mutex<HashMap<String, String>>>>>>;

#[cfg(feature = "proxy")]
#[derive(Clone)]
pub struct AppState {
    pub cache: Cache,
}

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[macro_export]
macro_rules! log_message {
    ($level:expr, $($arg:tt)*) => {
        match $level {
            $crate::LogLevel::Trace => {
                #[cfg(feature = "proxy")]
                {
                    tracing::trace!($($arg)*);
                }
                #[cfg(not(feature = "proxy"))]
                {
                    println!("[TRACE] {}", format!($($arg)*));
                }
            },
            $crate::LogLevel::Debug => {
                #[cfg(feature = "proxy")]
                {
                    tracing::debug!($($arg)*);
                }
                #[cfg(not(feature = "proxy"))]
                {
                    println!("[INFO] {}", format!($($arg)*));
                }
            },
            $crate::LogLevel::Info => {
                #[cfg(feature = "proxy")]
                {
                    tracing::info!($($arg)*);
                }
                #[cfg(not(feature = "proxy"))]
                {
                    println!("[INFO] {}", format!($($arg)*));
                }
            },
            $crate::LogLevel::Warn => {
                #[cfg(feature = "proxy")]
                {
                    tracing::warn!($($arg)*);
                }
                #[cfg(not(feature = "proxy"))]
                {
                    println!("[WARN] {}", format!($($arg)*));
                }
            },
            $crate::LogLevel::Error => {
                #[cfg(feature = "proxy")]
                {
                    tracing::error!($($arg)*);
                }
                #[cfg(not(feature = "proxy"))]
                {
                    println!("[ERROR] {}", format!($($arg)*));
                }
            },
        }
    };
}
