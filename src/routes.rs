#[cfg(feature = "proxy")]
use {
    crate::{
        parse::{add_http_prefix, get_rss_feed},
        AppState, FurssOptions,
    },
    axum::{
        extract::{Query, State},
        http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
        response::IntoResponse,
    },
};

#[cfg(feature = "proxy")]
/// # Panics
///
/// Will panic if `application/xml` is not a valid content-type
pub async fn handler(
    req_headers: HeaderMap,
    uri: axum::http::Uri,
    options: Query<FurssOptions>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let options2: FurssOptions = options.0;
    let mut headers = HeaderMap::new();
    let response = match req_headers.get(CONTENT_TYPE).map(HeaderValue::as_bytes) {
        Some(b"application/xml") => {
            let response = get_rss_feed(&add_http_prefix(uri.path()), &options2, &state).await;

            headers.insert(CONTENT_TYPE, "application/xml".parse().unwrap());

            response
        }

        _ => String::from("Hello, world!"),
    };

    (headers, response)
}
