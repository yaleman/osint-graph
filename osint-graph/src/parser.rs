use regex::Regex;

#[cfg(target = "wasm32")]
use gloo_console::debug;

static URL_REGEX: &str =
    r#"(?P<url>(http|ftp|https|mailto|gopher|)\:\/\/[a-z0-9A-Z\.\?\#\=\:\/]+)"#;

#[derive(Debug, Eq, PartialEq)]
pub enum ContentType {
    #[allow(dead_code)]
    Url(String),
    Unknown(String),
}

#[allow(dead_code)]
fn parse_input(input: &str) -> Result<ContentType, String> {
    // urls

    let url_matcher = Regex::new(URL_REGEX).expect("Failed to parse URL Regex?");

    if let Some(url) = url_matcher.captures(input) {
        #[cfg(target = "wasm32")]
        debug!(format!("Found a url! {:?}", url));
        return Ok(ContentType::Url(url["url"].to_string()));
    }

    Ok(ContentType::Unknown(input.to_string()))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_url() {
        use super::{parse_input, ContentType};

        let url =
            "asdfasdf https://www.example.com:1234/asdffadsf?foo=bar#asdfhh asdfasdf".to_string();
        let expected = "https://www.example.com:1234/asdffadsf?foo=bar#asdfhh".to_string();

        let res = parse_input(&url);
        dbg!(&res);
        assert_eq!(res.unwrap(), ContentType::Url(expected));
    }
}
