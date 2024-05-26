#[cfg(feature = "proxy")]
use {
    crate::{
        parse::{add_http_prefix, get_rss_feed},
        AppState, FurssOptions,
    },
    axum::{
        extract::{Query, State},
        http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
        response::{IntoResponse, Response},
    },
};

#[cfg(feature = "proxy")]
pub async fn handler(
    headers: HeaderMap,
    uri: axum::http::Uri,
    options: Query<FurssOptions>,
    State(state): State<AppState>,
) -> Response {
    let options2: FurssOptions = options.0;

    match headers.get(CONTENT_TYPE).map(HeaderValue::as_bytes) {
        Some(b"application/xml") => {
            let response =
                get_rss_feed(&add_http_prefix(uri.path()), &options2, Some(&state)).await;
            response.into_response()
        }

        _ => String::from("Hello, world!").into_response(),
    }
}
