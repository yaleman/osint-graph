//* Functionality to identify contents / nodes
//*

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub enum SocialNode {
    Facebook(String),
    Twitter(String),
    Instagram(String),
    Youtube(String),
    Tiktok(String),
    Reddit(String),
    // dis one be hard
    Mastodon(String),
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub enum UrlNode {
    Unknown,
    Social(SocialNode),
}

pub fn identify_url(input: &str) -> Result<UrlNode, String> {
    let url = url::Url::parse(input).expect("Failed to parse URL");

    let host: String = match url.host() {
        None => return Err(format!("No host in url: {}", input)),
        Some(val) => val.to_string(),
    };

    //insta
    if host == "instagram.com" || host.ends_with(".instagram.com") {
        println!("Instagram: {:?}", url);
        // todo: parse out username
        Ok(UrlNode::Social(SocialNode::Instagram(input.to_string())))
    } else if host == "twitter.com"
        || host == "x.com"
        || host.ends_with(".twitter.com")
        || host.ends_with(".x.com")
    {
        Ok(UrlNode::Social(SocialNode::Twitter(input.to_string())))
    } else if host == "tiktok.com" || host.ends_with(".twitter.com") {
        Ok(UrlNode::Social(SocialNode::Tiktok(input.to_string())))
    } else if host == "facebook.com" || host.ends_with(".facebook.com") {
        Ok(UrlNode::Social(SocialNode::Facebook(input.to_string())))
    } else if host == "reddit.com" || host.ends_with(".reddit.com") {
        Ok(UrlNode::Social(SocialNode::Reddit(input.to_string())))
    } else if host == "youtube.com" || host.ends_with(".youtube.com") {
        Ok(UrlNode::Social(SocialNode::Youtube(input.to_string())))
    } else {
        println!("Url: {:?}", url);
        Ok(UrlNode::Unknown)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_identify_url() {
        use super::*;

        let url = "https://www.instagram.com/yaleman13/";

        assert_eq!(
            identify_url(url).unwrap(),
            UrlNode::Social(SocialNode::Instagram(url.to_string()))
        );

        let url = "https://old.reddit.com/u/yaleman";

        assert_eq!(
            identify_url(url).unwrap(),
            UrlNode::Social(SocialNode::Reddit(url.to_string()))
        );
        let url = "https://www.facebook.com/profile.php?id=100064082793320";

        assert_eq!(
            identify_url(url).unwrap(),
            UrlNode::Social(SocialNode::Facebook(url.to_string()))
        );
    }
}
