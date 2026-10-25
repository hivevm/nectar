pub type FetchError = Box<dyn std::error::Error + Send + Sync>;

/// What a fetcher received for a URL.
#[derive(Debug, Clone, PartialEq)]
pub struct Fetched {
    /// The media type the server named, such as `image/png`; empty when it named none.
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// The port for images from the web. Adapters implement it, with the limits of time and size
/// their host allows; the core knows no adapter and makes no request of its own.
pub trait Fetcher: Send + Sync {
    fn fetch(&self, url: &str) -> Result<Fetched, FetchError>;
}
