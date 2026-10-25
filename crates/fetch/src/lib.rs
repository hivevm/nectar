//! Fetcher that downloads images from the web over HTTP, within limits of time and size.

use std::time::Duration;

use nectar_core::{FetchError, Fetched, Fetcher};

/// Most a single image may take, in time and size.
const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BYTES: u64 = 20 * 1024 * 1024;
const MAX_REDIRECTS: u32 = 5;

/// Fetches over HTTP and HTTPS with `ureq`.
pub struct Http {
    agent: ureq::Agent,
}

impl Http {
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .max_redirects(MAX_REDIRECTS)
            .build()
            .into();
        Self { agent }
    }
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for Http {
    fn fetch(&self, url: &str) -> Result<Fetched, FetchError> {
        let mut response = self.agent.get(url).call()?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()?;
        Ok(Fetched {
            content_type,
            bytes,
        })
    }
}

// The golden path's test pattern: an adapter tested against a real counterpart on loopback, the
// core's side against a fake behind the same port (nectar-core, download.rs).
// Verifies: ADR-0008
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Serves one HTTP response per connection, `count` times.
    fn serve(content_type: &'static str, body: &'static [u8], count: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(count) {
                let mut stream = stream.unwrap();
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        format!("http://{address}/picture.png")
    }

    #[test]
    fn fetches_the_body_and_its_media_type() {
        let url = serve("image/png", b"\x89PNG fake", 1);
        let fetched = Http::new().fetch(&url).unwrap();
        assert_eq!(fetched.content_type, "image/png");
        assert_eq!(fetched.bytes, b"\x89PNG fake");
    }

    #[test]
    fn reports_what_cannot_be_reached() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/gone.png", listener.local_addr().unwrap());
        drop(listener);
        assert!(Http::new().fetch(&url).is_err());
    }
}
