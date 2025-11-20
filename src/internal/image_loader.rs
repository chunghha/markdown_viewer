/*!
Image loader helpers

This module centralizes network fetching logic used by the image loading path.
It provides:

- `fetch_bytes_with_optional_png_fallback`:
  Fetches bytes from a URL and returns the raw bytes. The function logs
  status and content-type information and returns the raw body as a `Vec<u8>`.

- `png_fallback_url`:
  Utility to construct a server-side PNG fallback URL from an existing URL.
  (Matches the simple strategy used elsewhere: replace the first `?` with
  `.png?` or append `.png` if there is no query string.)

- `fetch_and_decode_image`:
  High-level helper that fetches a resource (remote or local) and attempts to
  decode it into an `image::DynamicImage`. The function:
  1) fetches remote bytes (or reads a local file),
  2) tries to decode as a raster image,
  3) if it looks like SVG, attempts SVG rasterization via crate helper,
  4) attempts a PNG fallback URL when available.

Notes:
- This module intentionally keeps fetching simple and returns raw bytes so
  callers may attempt to decode (e.g., as raster image or SVG) and then
  perform further fallback behavior if decoding fails.
*/

use anyhow::Result;
use reqwest::header::CONTENT_TYPE;
use tracing::{debug, info};

/// Fetch bytes from the given URL and return them as a Vec<u8>.
///
/// This function logs the HTTP status and Content-Type header when available.
/// It does not attempt to interpret or decode the bytes — callers should
/// decide how to treat the returned payload (raster decode, SVG rasterize, etc).
///
/// # Errors
///
/// Returns an error if the underlying HTTP request fails or the body cannot be
/// read into memory.
pub async fn fetch_bytes_with_optional_png_fallback(url: &str) -> Result<Vec<u8>, anyhow::Error> {
    // Perform a simple GET request. Use reqwest's convenience `get` for brevity.
    let resp = reqwest::get(url).await?;
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Read body into owned Vec<u8>
    let bytes = resp.bytes().await?.to_vec();

    debug!(
        "Fetched {} bytes from {} (status={}, ct={:?})",
        bytes.len(),
        url,
        status,
        content_type
    );

    Ok(bytes)
}

/// High-level helper: fetch (remote or local) and decode into an image::DynamicImage.
///
/// Strategy:
/// 1) If `path` starts with http:// or https://, fetch via HTTP and try to decode.
/// 2) If decode as raster succeeds, return it.
/// 3) If payload looks like an SVG (content starts with '<' or filename ends with .svg),
///    attempt crate::rasterize_svg_to_dynamic_image to rasterize into a DynamicImage.
/// 4) If that fails, attempt a server-side PNG fallback (replace `?` with `.png?` or append `.png`)
///    and try decoding that response as a raster image.
/// 5) If `path` is a local filesystem path, use `image::open`.
pub async fn fetch_and_decode_image(path: &str) -> Result<image::DynamicImage, anyhow::Error> {
    if path.starts_with("http://") || path.starts_with("https://") {
        info!("Starting remote image download: {}", path);

        // Primary fetch
        let primary_bytes = fetch_bytes_with_optional_png_fallback(path).await?;

        // Try decode as raster
        match image::load_from_memory(&primary_bytes) {
            Ok(img) => Ok(img),
            Err(_orig) => {
                // Determine if it looks like SVG by content or filename
                let looks_like_svg =
                    primary_bytes.starts_with(b"<") || path.to_lowercase().ends_with(".svg");
                if looks_like_svg {
                    match crate::rasterize_svg_to_dynamic_image(&primary_bytes) {
                        Ok(img) => Ok(img),
                        Err(e) => {
                            debug!("SVG rasterization failed for {}: {}", path, e);
                            // fallthrough to PNG fallback attempt
                            let png_url = png_fallback_url(path);
                            info!("Attempting PNG fallback for {}: {}", path, png_url);
                            let fallback_bytes =
                                fetch_bytes_with_optional_png_fallback(&png_url).await?;
                            let img2 = image::load_from_memory(&fallback_bytes)
                                .map_err(anyhow::Error::new)?;
                            Ok(img2)
                        }
                    }
                } else {
                    // Not SVG and raster decode failed: try PNG fallback
                    let png_url = png_fallback_url(path);
                    info!("Attempting PNG fallback for {}: {}", path, png_url);
                    let fallback_bytes = fetch_bytes_with_optional_png_fallback(&png_url).await?;
                    let img2 =
                        image::load_from_memory(&fallback_bytes).map_err(anyhow::Error::new)?;
                    Ok(img2)
                }
            }
        }
    } else {
        // Local file
        info!("Loading local image: {}", path);
        let img = image::open(path)?;
        Ok(img)
    }
}

/// Given an original URL, return a server-side PNG fallback URL.
///
/// Strategy:
/// - If the URL contains a `?` (query), replace the first `?` with `.png?`.
///   Example: `/600x400?text=Hello` -> `/600x400.png?text=Hello`
/// - Otherwise append `.png` to the path portion:
///   Example: `/800` -> `/800.png`
///
/// This mirrors the fallback strategy used by various placeholder services.
pub fn png_fallback_url(original: &str) -> String {
    if original.contains('?') {
        original.replacen('?', ".png?", 1)
    } else {
        format!("{}.png", original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple unit tests for png_fallback_url
    #[test]
    fn png_fallback_with_query() {
        let in_url = "https://placehold.co/600x400?text=Hello+World";
        let out = png_fallback_url(in_url);
        assert_eq!(
            out,
            "https://placehold.co/600x400.png?text=Hello+World".to_string()
        );
    }

    #[test]
    fn png_fallback_without_query() {
        let in_url = "https://placehold.co/800";
        let out = png_fallback_url(in_url);
        assert_eq!(out, "https://placehold.co/800.png".to_string());
    }
}
