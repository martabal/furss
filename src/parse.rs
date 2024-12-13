use core::str;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::Cursor,
    sync::Arc,
};

use futures::{stream, StreamExt};
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use reqwest::{Client, Url};
use tokio::sync::Mutex;

use crate::{log_message, AppState, FurssOptions, LogLevel};

#[must_use]
pub fn add_http_prefix(mut url: &str) -> String {
    url = url.trim_start_matches('/');
    if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("http://{url}")
    } else {
        url.to_string()
    }
}

fn parse_rss_feed(content: &str) -> Vec<String> {
    let mut reader = Reader::from_str(content);
    let mut buf = Vec::new();

    let mut urls: Vec<String> = Vec::new();
    let mut in_item = false;

    loop {
        match &reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"item" => in_item = true,
                b"link" if in_item => {
                    let link = reader
                        .read_text(e.name())
                        .expect("Cannot decode text value");
                    urls.push(link.to_string());
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"item" {
                    in_item = false;
                }
            }
            _ => (),
        }
        buf.clear();
    }
    urls
}

fn add_content_to_item(content: &str, cache: &HashMap<String, String>) -> String {
    let mut reader = Reader::from_str(content);

    let mut temp_content: VecDeque<Event> = VecDeque::new();

    let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
    let mut url: String = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"item" => {
                    temp_content.push_back(Event::Start(e.clone()));
                }
                b"link" => {
                    if temp_content.is_empty() {
                        writer
                            .write_event(Event::Start(e.clone()))
                            .expect("Failed to write start tag");
                    } else {
                        temp_content.push_back(Event::Start(e.clone()));
                        let link = reader
                            .clone()
                            .read_text(e.name())
                            .expect("Cannot decode text value");
                        url = link.to_string();
                    }
                }

                _ => {
                    if temp_content.is_empty() {
                        writer
                            .write_event(Event::Start(e.clone()))
                            .expect("Failed to write start tag");
                    } else {
                        temp_content.push_back(Event::Start(e.clone()));
                    }
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"item" {
                    if !url.is_empty() {
                        let content = cache.get(&url).cloned();

                        if let Some(review) = content {
                            while let Some(event) = temp_content.pop_front() {
                                writer.write_event(event).expect("Failed to write end tag");
                            }
                            writer
                                .write_event(Event::Start(BytesStart::new("ns0:encoded")))
                                .expect("Failed to write end tag");
                            writer
                                .write_event(Event::Text(BytesText::new(&review)))
                                .expect("Failed to write end tag");
                            writer
                                .write_event(Event::End(BytesEnd::new("ns0:encoded")))
                                .expect("Failed to write end tag");
                            writer
                                .write_event(Event::End(e.clone()))
                                .expect("Failed to write end tag");
                        }
                        url = String::new();
                    }

                    temp_content = VecDeque::new();
                } else if temp_content.is_empty() {
                    writer
                        .write_event(Event::End(e.clone()))
                        .expect("Failed to write end tag");
                } else {
                    temp_content.push_back(Event::End(e.clone()));
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if temp_content.is_empty() {
                    writer.write_event(e).unwrap();
                } else {
                    temp_content.push_back(e);
                }
            }

            Err(error) => panic!("Error at position {error}"),
        }
    }
    String::from_utf8(writer.into_inner().into_inner()).unwrap()
}

async fn embellish_feed(
    content: &str,
    options: &FurssOptions,
    cache: &mut HashMap<String, String>,
) -> String {
    let urls = parse_rss_feed(content);

    let mut url_requests: Vec<String> = match options.full {
        Some(true) => urls,
        _ => urls
            .iter()
            .take(options.number_items.map_or(10, std::convert::Into::into))
            .cloned()
            .collect(),
    };

    let mut articles: HashMap<String, String> = HashMap::new();

    for (key, value) in url_requests
        .iter()
        .filter_map(|key| cache.get(key).map(|value| (key.clone(), value.clone())))
    {
        articles.insert(key, value);
    }

    // Empty cache
    url_requests.retain(|x| !cache.contains_key(x));

    // Remove cached articles
    let vec_set: HashSet<_> = url_requests.iter().cloned().collect();
    cache.retain(|key, _| vec_set.contains(key));

    let arc_articles: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(articles));

    let client = Client::new();

    let bodies = stream::iter(url_requests.clone())
        .map(|url| {
            let client = &client;
            async move {
                let resp = client.get(&url).send().await?;
                let body = resp.bytes().await?;
                Ok::<_, reqwest::Error>((url, body))
            }
        })
        .buffer_unordered(8);

    bodies
        .for_each(|result| async {
            match result {
                Ok((url, body)) => {
                    log_message!(
                        LogLevel::Trace,
                        "{}",
                        format!("RESPONSE: {} bytes from {}", body.len(), url)
                    );

                    let body_string = str::from_utf8(&body).unwrap();
                    if let Some(content) = extract_content(body_string) {
                        let mut oulou = arc_articles.lock().await;
                        oulou.insert(url, content);
                    }
                }
                Err(e) => log_message!(LogLevel::Warn, "ERROR downloading {e}"),
            }
        })
        .await;

    let cloned_articles = arc_articles.lock().await.clone();

    for (url, article) in cloned_articles.clone() {
        cache.insert(url, article);
    }

    add_content_to_item(content, &cloned_articles)
}

fn extract_content(content: &str) -> Option<String> {
    let dom = tl::parse(content, tl::ParserOptions::default()).unwrap();
    let mut filtered_nodes = dom
        .nodes()
        .iter()
        .filter(|node| node.as_tag().map_or(true, |tag| tag.name() != "script"));

    // Find the article tag among the filtered nodes
    let article_node =
        filtered_nodes.find(|node| node.as_tag().map_or(false, |tag| tag.name() == "article"));

    Some(
        std::str::from_utf8(article_node?.as_tag()?.raw().as_bytes())
            .ok()?
            .to_owned(),
    )
}

/// # Panics
///
/// Will panic if url is not a valid url
pub async fn get_rss_feed(url: &str, options: &FurssOptions, state: &AppState) -> String {
    let mut rss_url = Url::parse(url).unwrap();
    rss_url.query_pairs_mut().clear();
    let body = match &options.flaresolverr {
        Some(flaresolverr_url) => {
            let mut map = HashMap::new();
            map.insert("cmd", "request.get");
            map.insert("url", rss_url.as_str());
            let client = Client::new();
            let response = client
                .post(flaresolverr_url)
                .json(&map)
                .send()
                .await
                .unwrap();

            response.text().await.unwrap()
        }
        None => reqwest::get(rss_url).await.unwrap().text().await.unwrap(),
    };

    let mut cache = state.cache.lock().await.get(url).map_or_else(
        || HashMap::<String, String>::new(),
        std::clone::Clone::clone,
    );

    let response = embellish_feed(&body, options, &mut cache).await;
    state.cache.lock().await.insert(url.to_string(), cache);

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testadd_http_prefix_no_prefix() {
        let url = "example.com";
        let expected = "http://example.com";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_http_prefix() {
        let url = "http://example.com";
        let expected = "http://example.com";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_https_prefix() {
        let url = "https://example.com";
        let expected = "https://example.com";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_leading_slash() {
        let url = "/example.com";
        let expected = "http://example.com";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_multiple_leading_slashes() {
        let url = "///example.com";
        let expected = "http://example.com";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_complex_url_no_prefix() {
        let url = "example.com/path?query=1";
        let expected = "http://example.com/path?query=1";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_complex_url_with_http() {
        let url = "http://example.com/path?query=1";
        let expected = "http://example.com/path?query=1";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn testadd_http_prefix_complex_url_with_https() {
        let url = "https://example.com/path?query=1";
        let expected = "https://example.com/path?query=1";
        assert_eq!(add_http_prefix(url), expected);
    }

    #[test]
    fn test_extract_content_without_script_tags() {
        let content = r"<html><head><title>Test</title></head><body><article><h1>Article Title</h1><p>Article content goes here.</p></article></body></html>";

        assert_eq!(
            extract_content(content).unwrap(),
            "<article><h1>Article Title</h1><p>Article content goes here.</p></article>"
        );
    }

    #[test]
    fn test_extract_content_with_script_tags() {
        let content = r#"<html><head><title>Test</title></head><body><script>console.log("This is a script")<script><article><h1>Article Title</h1><p>Article content goes here.</p></article></body></html>"#;

        assert_eq!(
            extract_content(content).unwrap(),
            "<article><h1>Article Title</h1><p>Article content goes here.</p></article>"
        );
    }

    #[test]
    fn test_extract_content_no_article_tag() {
        let content = r"<html><head><title>Test</title></head><body><div><h1>Another Title</h1><p>Some content</p></div></body><html>";

        assert_eq!(extract_content(content), None);
    }

    #[test]
    fn test_add_content_to_item() {
        let content = r#"<rss version="2.0"><channel><title>Test</title><link>https://test.com/</link><description>RSS to test</description><language>en-us</language><item><title>First article</title><link>https://example.org</link><description>This is the description of example.org</description><pubDate>Sun, 26 May 2024 10:00:00 -0400</pubDate><dc:creator>martabal</dc:creator></item><item><title>Second article</title><link>https://not.in.hashmap.com</link><description>This is the description of not.in.hashmap.com</description><pubDate>Sun, 26 May 2024 09:00:00 -0400</pubDate><dc:creator>martabal</dc:creator></item></channel></rss>"#;

        let expect = r#"<rss version="2.0"><channel><title>Test</title><link>https://test.com/</link><description>RSS to test</description><language>en-us</language><item><title>First article</title><link>https://example.org</link><description>This is the description of example.org</description><pubDate>Sun, 26 May 2024 10:00:00 -0400</pubDate><dc:creator>martabal</dc:creator><ns0:encoded>Content of example.org</ns0:encoded></item></channel></rss>"#;
        let mut cache = HashMap::new();

        // Review some books.
        cache.insert(
            "https://example.org".to_string(),
            "Content of example.org".to_string(),
        );
        assert_eq!(add_content_to_item(content, &cache), expect);
    }
}
