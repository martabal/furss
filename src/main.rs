use axum::handler::Handler;
use furss::{APP_DEFAULT_PORT, APP_NAME, APP_VERSION};

use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

#[cfg(feature = "proxy")]
use {
    dotenvy::dotenv,
    furss::{routes::handler, AppState, APP_PORT},
    std::net::SocketAddr,
    tracing::{info, warn},
    tracing_subscriber::{filter::LevelFilter, EnvFilter},
};

#[tokio::main]
async fn main() {
    APP_NAME.set(env!("CARGO_PKG_NAME").to_string()).unwrap();
    APP_VERSION
        .set(env!("CARGO_PKG_VERSION").to_string())
        .unwrap();

    #[cfg(feature = "proxy")]
    dotenv().ok();

    let log_level_str = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    #[cfg(feature = "proxy")]
    {
        let filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env()
            .unwrap()
            .add_directive(
                format!("{}={log_level_str}", APP_NAME.get().unwrap())
                    .parse()
                    .unwrap(),
            );

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .init();

        let state = AppState {
            cache: Arc::new(Mutex::new(HashMap::new())),
        };
        let app_port = env::var("APP_PORT").map_or_else(
            |_| {
                warn!(
                    "Environment variable not found, defaulting to default ({})",
                    APP_DEFAULT_PORT
                );
                APP_DEFAULT_PORT
            },
            |val| val.parse().expect("Provided port is not a valid u16"),
        );
        APP_PORT.set(app_port).unwrap();

        let address = SocketAddr::from(([0, 0, 0, 0], *APP_PORT.get().unwrap()));
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();

        info!(
            "Starting {} version {}",
            APP_NAME.get().unwrap(),
            APP_VERSION.get().unwrap()
        );
        axum::serve(listener, handler.with_state(state).into_make_service())
            .await
            .unwrap();
    }
}
