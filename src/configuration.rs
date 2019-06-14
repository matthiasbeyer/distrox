use crate::types::util::MimeType;

// use chrono::Duration;

/// Configuration read from a configuration file
#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    #[serde(rename = "ipfs-api-url")]
    /// The URL of the API
    api_url: String,

    #[serde(rename = "ipfs-api-port")]
    /// The Port of the API
    api_port: u16,

    #[serde(rename = "autoserve-chains")]
    /// Whether to automatically "ipfs pin" chain objects
    autoserve_chains: bool,

    #[serde(rename = "autoserve-text-posts")]
    /// Whether to automatically "ipfs pin" foreign posts if their content is text
    autoserve_text_posts: bool,

    #[serde(rename = "serve-blocked")]
    /// Whether to serve content/chains from blocked profiles
    serve_blocked: bool,

    #[serde(rename = "autoserve-followed")]
    /// Whether to automatically "ipfs pin" followed profiles
    autoserve_followed: bool,

    #[serde(rename = "max-autoload-per-post")]
    /// Default amount of bytes which are loaded for each post
    max_autoload_per_post: usize,

    #[serde(rename = "autoserve-blacklist")]
    /// List of Mimetypes which should not be served
    autoserve_blacklist: Vec<MimeType>,

    #[serde(rename = "autoserve-whitelist")]
    /// List of Mimetypes which can be served
    autoserve_whitelist: Vec<MimeType>,

    // #[serde(rename = "merge-timeout")]
    // /// Timeout before merge should be attempted
    // merge_timeout: Duration,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            api_url               : String::from("127.0.0.1"),
            api_port              : 5001,
            autoserve_chains      : true,
            autoserve_text_posts  : true,
            serve_blocked         : false,
            autoserve_followed    : true,
            max_autoload_per_post : 1024 * 1024,
            autoserve_blacklist   : Vec::new(),
            autoserve_whitelist   : Vec::new(),
            // merge_timeout         : Duration::minutes(15),
        }
    }
}

impl Configuration {
    pub fn api_url(&self) -> &String {
        &self.api_url
    }

    pub fn api_port(&self) -> u16 {
        self.api_port
    }

    pub fn autoserve_chains(&self) -> bool {
        self.autoserve_chains
    }

    pub fn autoserve_text_posts(&self) -> bool {
        self.autoserve_text_posts
    }

    pub fn serve_blocked(&self) -> bool {
        self.serve_blocked
    }

    pub fn autoserve_followed(&self) -> bool {
        self.autoserve_followed
    }

    pub fn max_autoload_per_post(&self) -> usize {
        self.max_autoload_per_post
    }

    pub fn autoserve_blacklist(&self) -> &Vec<MimeType> {
        &self.autoserve_blacklist
    }

    pub fn autoserve_whitelist(&self) -> &Vec<MimeType> {
        &self.autoserve_whitelist
    }
}
