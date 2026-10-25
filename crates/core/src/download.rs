//! Images from the web and from `data:` URIs become part of the documentation: the download
//! pass fetches them once and keeps them with the document, so every renderer can include them
//! like local images.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use crate::diagnostic::Diagnostic;
use crate::model::{
    Document, Inline, Target, block_inlines_mut, visit_blocks_mut, visit_inlines_mut,
};

/// Most a single image may take, in time and size.
const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BYTES: u64 = 20 * 1024 * 1024;
const MAX_REDIRECTS: u32 = 5;

/// Where downloaded images are kept, below the document's output directory.
pub(crate) const DIRECTORY: &str = "_remote";

#[derive(Debug, Clone)]
pub(crate) struct Download {
    pub extension: String,
    pub bytes: Vec<u8>,
}

/// Fetches images, each URL once per project.
pub(crate) struct Downloader {
    offline: bool,
    cache: Mutex<HashMap<String, Result<Download, String>>>,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            offline: false,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    /// The image at `url`, or why it is not available. `None` when nothing is to be fetched:
    /// the URL is no image source, or the project is offline.
    fn get(&self, url: &str) -> Option<Result<Download, String>> {
        if let Some(data) = url.strip_prefix("data:") {
            return Some(decode_data_uri(data));
        }
        let lower = url.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) || self.offline {
            return None;
        }
        if let Some(result) = self.cache.lock().unwrap().get(url) {
            return Some(result.clone());
        }
        let result = fetch(url);
        self.cache
            .lock()
            .unwrap()
            .insert(url.to_string(), result.clone());
        Some(result)
    }
}

fn fetch(url: &str) -> Result<Download, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .max_redirects(MAX_REDIRECTS)
        .build()
        .into();
    let mut response = agent.get(url).call().map_err(|e| e.to_string())?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let Some(extension) = image_extension(&content_type, url) else {
        return Err(format!("the server sent '{content_type}', not an image"));
    };
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    Ok(Download { extension, bytes })
}

/// The file extension for an image media type; `None` for anything else.
fn image_extension(content_type: &str, url: &str) -> Option<String> {
    let media = content_type.split(';').next().unwrap_or_default().trim();
    let subtype = media.strip_prefix("image/")?;
    let extension = match subtype {
        "jpeg" | "jpg" | "pjpeg" => "jpg",
        "svg+xml" => "svg",
        "x-icon" | "vnd.microsoft.icon" => "ico",
        "png" | "gif" | "webp" | "avif" | "bmp" | "tiff" => subtype,
        _ => Path::new(url.split(['?', '#']).next().unwrap_or_default())
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("img"),
    };
    Some(extension.to_string())
}

/// `image/png;base64,iVBOR…`
fn decode_data_uri(data: &str) -> Result<Download, String> {
    let (header, payload) = data.split_once(',').ok_or("malformed data URI")?;
    let Some(media) = header.strip_suffix(";base64") else {
        return Err("only base64 data URIs are supported".to_string());
    };
    let extension = image_extension(media, "").ok_or("the data URI holds no image")?;
    let bytes = decode_base64(payload).ok_or("the data URI holds invalid base64")?;
    Ok(Download { extension, bytes })
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len() * 3 / 4);
    let (mut buffer, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(bytes)
}

/// Turns images from the web and from `data:` URIs into assets of the document. An image that
/// cannot be fetched keeps its URL and is reported.
pub(crate) fn download_images(
    document: &mut Document,
    downloader: &Downloader,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let prefix = PathBuf::from(&document.path);
    let mut assets = std::mem::take(&mut document.assets);
    let mut downloads = std::mem::take(&mut document.downloads);
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        let origin = block.origin.clone();
        block_inlines_mut(block, &mut |inlines| {
            visit_inlines_mut(inlines, &mut |inline| {
                let Inline::Image(image) = inline else { return };
                let Target::External(url) = &image.target else {
                    return;
                };
                match downloader.get(url) {
                    None => {}
                    Some(Ok(download)) => {
                        let mut hasher = DefaultHasher::new();
                        url.hash(&mut hasher);
                        let name = format!("{:016x}.{}", hasher.finish(), download.extension);
                        let source = Path::new(DIRECTORY).join(&name);
                        assets.insert(source.clone(), prefix.join(DIRECTORY).join(&name));
                        downloads.insert(source.clone(), download.bytes);
                        image.target = Target::Asset(source);
                    }
                    Some(Err(message)) => {
                        let shown: String = url.chars().take(80).collect();
                        diagnostics.push(Diagnostic::warning(
                            format!("cannot include the image {shown}: {message}"),
                            Some(origin.clone()),
                        ));
                    }
                }
            });
        });
    });
    document.assets = assets;
    document.downloads = downloads;
}

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
    fn downloads_images_once() {
        let url = serve("image/png", b"\x89PNG fake", 1);
        let downloader = Downloader::new();
        let first = downloader.get(&url).unwrap().unwrap();
        assert_eq!(first.extension, "png");
        assert_eq!(first.bytes, b"\x89PNG fake");
        // The server answers only once; the second call comes from the cache.
        assert_eq!(downloader.get(&url).unwrap().unwrap().bytes, first.bytes);
    }

    #[test]
    fn refuses_what_is_no_image() {
        let url = serve("text/html", b"<html></html>", 1);
        let error = Downloader::new().get(&url).unwrap().unwrap_err();
        assert!(error.contains("not an image"), "{error}");
    }

    #[test]
    fn stays_offline_when_asked() {
        let mut downloader = Downloader::new();
        downloader.set_offline(true);
        assert!(downloader.get("https://example.org/a.png").is_none());
        assert!(
            downloader
                .get("data:image/gif;base64,R0lGOA==")
                .unwrap()
                .is_ok()
        );
    }

    #[test]
    fn decodes_data_uris() {
        let download = decode_data_uri("image/png;base64,iVBORw0KGgo=").unwrap();
        assert_eq!(download.extension, "png");
        assert_eq!(download.bytes, b"\x89PNG\r\n\x1a\n");
        assert!(decode_data_uri("text/plain;base64,aGk=").is_err());
        assert!(decode_data_uri("image/png,raw").is_err());
    }
}
