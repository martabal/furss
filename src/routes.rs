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
/// # Errors
///
/// Will return `Err` if there's an error when getting the rss feed
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
            (get_rss_feed(&add_http_prefix(uri.path()), &options2, &state).await).map_or_else(
                |_| {
                    Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Error fetching RSS feed".to_string(),
                    ))
                },
                |response| {
                    "application/xml".parse::<HeaderValue>().map_or_else(
                        |_| {
                            Err((
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "Error setting Content-Type header".to_string(),
                            ))
                        },
                        |header_value| {
                            headers.insert(CONTENT_TYPE, header_value);
                            Ok((headers, response))
                        },
                    )
                },
            )
        }

        _ => Ok((headers, String::from("Hello, world!"))),
    };

    response
}
