use crate::types::util::MimeType;

// use chrono::Duration;

/// Configuration read from a configuration file
#[derive(Serialize, Deserialize, Debug, AddGetterVal)]
pub struct Configuration {
    #[serde(rename = "ipfs-api-url")]
    #[get_val]
    /// The URL of the API
    api_url: String,

    #[serde(rename = "ipfs-api-port")]
    #[get_val]
    /// The Port of the API
    api_port: u16,

    #[serde(rename = "autoserve-chains")]
    #[get_val]
    /// Whether to automatically "ipfs pin" chain objects
    autoserve_chains: bool,

    #[serde(rename = "autoserve-text-posts")]
    #[get_val]
    /// Whether to automatically "ipfs pin" foreign posts if their content is text
    autoserve_text_posts: bool,

    #[serde(rename = "serve-blocked")]
    #[get_val]
    /// Whether to serve content/chains from blocked profiles
    serve_blocked: bool,

    #[serde(rename = "autoserve-followed")]
    #[get_val]
    /// Whether to automatically "ipfs pin" followed profiles
    autoserve_followed: bool,

    #[serde(rename = "max-autoload-per-post")]
    #[get_val]
    /// Default amount of bytes which are loaded for each post
    max_autoload_per_post: usize,

    #[serde(rename = "autoserve-blacklist")]
    #[get_val]
    /// List of Mimetypes which should not be served
    autoserve_blacklist: Vec<MimeType>,

    #[serde(rename = "autoserve-whitelist")]
    #[get_val]
    /// List of Mimetypes which can be served
    autoserve_whitelist: Vec<MimeType>,

    // #[serde(rename = "merge-timeout")]
    // #[get_val]
    // /// Timeout before merge should be attempted
    // merge_timeout: Duration,
    //

    /// Name under which to provide the local device. E.G.
    /// Some("/ipfs/QmVrLsEDn27sScp3k23sgZNefVTjSAL3wpgW1iWPi4MgoY")
    ///
    /// If none, one will be generated and set
    #[serde(rename = "device_name")]
    #[get_val]
    device_name: Option<String>,

    /// Key to sign stuff that comes from this device.
    ///
    /// Create by using `ipfs key gen <name>`
    #[serde(rename = "devicekey")]
    #[get_val]
    device_key: String,

    /// Devices for the profile
    /// E.G:
    /// ["/ipfs/QmVrLsEDn27sScp3k23sgZNefVTjSAL3wpgW1iWPi4MgoY"]
    #[serde(rename = "profiles")]
    #[get_val]
    devices: Vec<String>,
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
            device_name           : None,
            device_key            : None,
            devices               : Vec::new(),
        }
    }
}

