// TODO: do tests
pub struct TikTokUser {
    // pub id: String,
    pub username: String,
    // pub nickname: String,
    // pub avatar: String,
    // pub bio: String,
    // pub followers: i32,
    // pub following: i32,
    // pub likes: i32,
    // pub videos: i32,
    // pub verified: bool,
    // pub private: bool,
    // pub blocked: bool,
    // pub country: String,
    // pub region: String,
    // pub city: String,
}

impl TikTokUser {
    pub fn url(&self) -> String {
        let _url = "https://www.tiktok.com/api/user/detail/";
        let time = chrono::Utc::now().timestamp();

        let _params = [
        ("WebIdLastTime", time.to_string().as_str()),
        ("aid", "1988"),
        ("app_language", "en"),
        ("app_name", "tiktok_web"),
        ("browser_language", "en-AU"),
        ("browser_name", "Mozilla"),
        ("browser_online", "true"),
        ("browser_platform", "MacIntel"),
        ("browser_version", "5.0%20%28Macintosh%3B%20Intel%20Mac%20OS%20X%2010_15_7%29%20AppleWebKit%2F605.1.1520%28KHTML%2C%20like%20Gecko%29%20Version%2F17.2.1%20Safari%2F605.1.15"),
        ("channel", "tiktok_web"),
        ("cookie_enabled", "true"),
        // ("device_id", "7333968473036801543"), // maybe make this a random number
        ("device_platform", "web_pc"),
        ("focus_state","true"),
        ("from_page", "user"),
        ("history_len", "2"),
        ("is_fullscreen", "false"),
        ("is_page_visible", "true"),
        ("language", "en"),
        ("os", "mac"),
        ("priority_region", "" ),
        ("referer", ""),
        ("region", "AU"),
        ("screen_height", "982"),
        ("screen_width", "1512"),
        // ("secUid", "MS4wLjABAAAAOGwQju9GJb4TERHHfJ3PBKI8IEzTqBEjHqJfxRg6oSwAaU3DcXKszCp3AaVZgeWs"),
        ("tz_name","Australia%2FBrisbane"),
        ("uniqueId",self.username.as_str()),
        ("webcast_language","en"),
        // ("msToken","bb6ArFE5IGo-0wiq7lU1_lcQz5VIu-W3CYxkKZrfpnzboORF4a3Q8oYcFWW_thMXMmJtLWgslROVFjaKlq-p7zGZTHgyBwshzWwOs-IAVNEk8L5v2vr9JkbRbKtN"),
        // ("X-Bogus","DFSzsIVO9HiANyWftq/GmU9WcBjT"),
        // ("_signature","_02B4Z6wo00001TTbkYgAAIDBNNuRizkvIPk02ZUAACj047"),
    ];

        let _headers = [
            ("Accept", "*/*"),
            ("Accept-Language", "en-AU,en;q=0.9"),
            ("Cache-Control", "no-cache"),
            ("Pragma", "no-cache"),
            (
                "referrer",
                &format!("https://www.tiktok.com/@{}", self.username),
            ),
        ];
        let _method = "GET";
        todo!()
    }
}

// {
//     "cache": "default",
//     "credentials": "include",

//     "method": "GET",
//     "mode": "cors",
//     "redirect": "follow",
//     "referrer": "https://www.tiktok.com/@username?_t=8jleV3SJSs6&_r=1",
//     "referrerPolicy": "strict-origin-when-cross-origin"
// })
