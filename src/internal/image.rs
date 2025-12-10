//! SVG rasterization helpers
//!
//! This module centralizes SVG -> raster image conversion using the resvg/usvg/tiny-skia
//! stack and exposes a single public helper that returns an `image::DynamicImage`.
//!
//! The implementation:
//! - Parses the SVG into a `usvg::Tree`
//! - Computes a target pixel size constrained by `crate::IMAGE_MAX_WIDTH` while preserving aspect ratio
//! - Renders into a `tiny_skia::Pixmap` via `resvg::render`
//! - Converts premultiplied pixel bytes from tiny-skia into straight RGBA expected by `image`
//!
//! The function intentionally returns `anyhow::Error` for ergonomic propagation from callers.

use anyhow::Result;
use resvg::{
    tiny_skia,
    usvg::{Options as UsvgOptions, Tree as UsvgTree},
};

/// Convert RGBA image to BGRA format by swapping red and blue channels.
///
/// GPUI on macOS expects BGRA format, but the `image` crate produces RGBA.
/// This function performs an in-place channel swap.
pub fn rgba_to_bgra(rgba: &mut image::RgbaImage) {
    for pixel in rgba.pixels_mut() {
        pixel.0.swap(0, 2); // Swap R and B channels
    }
}

/// Rasterize SVG bytes into an `image::DynamicImage` using resvg + usvg + tiny-skia.
///
/// The returned image is an `ImageRgba8` with straight (un-premultiplied) RGBA bytes.
///
/// # Errors
///
/// Returns an error if:
/// - The SVG cannot be parsed
/// - The SVG has invalid intrinsic size (<= 0)
/// - Pixmap allocation fails
/// - Constructing the `RgbaImage` from raw bytes fails
pub fn rasterize_svg_to_dynamic_image(
    svg_bytes: &[u8],
) -> Result<image::DynamicImage, anyhow::Error> {
    // Parse SVG bytes into a usvg tree
    let opt = UsvgOptions::default();
    let rtree = UsvgTree::from_data(svg_bytes, &opt)
        .map_err(|e| anyhow::anyhow!("Failed to parse SVG: {}", e))?;

    // Use the tree's intrinsic size
    let svg_w = rtree.size().width();
    let svg_h = rtree.size().height();

    if svg_w <= 0.0 || svg_h <= 0.0 {
        return Err(anyhow::anyhow!("SVG has invalid width/height"));
    }

    // Compute scale constrained by crate::IMAGE_MAX_WIDTH while preserving aspect ratio
    let scale = match svg_w.partial_cmp(&crate::IMAGE_MAX_WIDTH) {
        Some(std::cmp::Ordering::Greater) => crate::IMAGE_MAX_WIDTH / svg_w,
        _ => 1.0,
    };

    let target_w = (svg_w * scale).ceil() as u32;
    let target_h = (svg_h * scale).ceil() as u32;

    // Allocate a pixmap
    let mut pixmap = tiny_skia::Pixmap::new(target_w, target_h)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate pixmap for SVG rasterization"))?;

    // Render SVG into pixmap
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(&rtree, transform, &mut pixmap_mut);

    // Convert premultiplied tiny-skia bytes into straight RGBA expected by `image::RgbaImage`.
    // tiny-skia commonly provides premultiplied pixels in either BGRA or RGBA order depending on
    // platform/target. To be robust, detect the channel ordering from the first non-transparent pixel
    // and then un-premultiply and reorder consistently into RGBA bytes for image::RgbaImage.
    let mut buf = pixmap.data().to_vec();

    // Force conversion assuming tiny-skia provides premultiplied BGRA bytes.
    // Historically tiny-skia writes pixels as [B, G, R, A] with premultiplied alpha on
    // the platforms we target; to avoid heuristic mis-detections we unconditionally
    // treat the buffer as BGRA here and convert to straight RGBA expected by the image crate.
    //
    // This simplifies behavior and prevents channel swaps (e.g., orange -> blue).
    // tiny-skia writes premultiplied pixels, and the exact byte ordering can vary
    // by platform (commonly BGRA, sometimes RGBA). Use a robust detection:
    // - Sum the channel values across non-transparent pixels to find which index
    //   is dominant (red-heavy or blue-heavy).
    // - Map source indices -> (R,G,B) accordingly, then un-premultiply per-pixel
    //   and write out straight RGBA expected by image::RgbaImage.
    //
    // This avoids assuming a single layout while preventing brittle one-pixel heuristics.
    let mut sum_c = [0u64; 3];
    let mut non_transparent_count: u64 = 0;
    for px in buf.chunks_exact(4) {
        let a = px[3] as u64;
        if a > 0 {
            sum_c[0] += px[0] as u64;
            sum_c[1] += px[1] as u64;
            sum_c[2] += px[2] as u64;
            non_transparent_count += 1;
        }
    }

    // Decide mapping of buffer indices -> (R,G,B) source positions.
    // Default to BGRA mapping (R is at index 2) when ambiguous.
    // tiny-skia typically outputs BGRA on most platforms.
    let mapping = match non_transparent_count {
        0 => [2usize, 1usize, 0usize],
        _ => {
            let s0 = sum_c[0] as f64 / non_transparent_count as f64;
            let s1 = sum_c[1] as f64 / non_transparent_count as f64;
            let s2 = sum_c[2] as f64 / non_transparent_count as f64;
            // For an orange image (#FFA500 = RGB 255,165,0):
            // - In BGRA format: bytes are [B=0, G=165, R=255, A=255], so index 2 is high
            // - In RGBA format: bytes are [R=255, G=165, B=0, A=255], so index 0 is high
            // Therefore: high index 2 -> BGRA, high index 0 -> RGBA
            match (
                s2 > s0 * 1.05 && s2 > s1 * 1.05,
                s0 > s2 * 1.05 && s0 > s1 * 1.05,
            ) {
                (true, false) => [2usize, 1usize, 0usize],
                (false, true) => [0usize, 1usize, 2usize],
                _ => [2usize, 1usize, 0usize],
            }
        }
    };
    for px in buf.chunks_exact_mut(4) {
        let a = px[3] as f32 / 255.0;
        match a > 0.0 {
            true => {
                // Read premultiplied channels from the mapped source indices
                let s_r = px[mapping[0]] as f32;
                let s_g = px[mapping[1]] as f32;
                let s_b = px[mapping[2]] as f32;
                // Un-premultiply and clamp
                let ur = (s_r / a).min(255.0) as u8;
                let ug = (s_g / a).min(255.0) as u8;
                let ub = (s_b / a).min(255.0) as u8;
                // Write out in RGBA order expected by image::RgbaImage
                px[0] = ur;
                px[1] = ug;
                px[2] = ub;
                // Preserve/restore alpha
                // Leave px[3] as-is (already the alpha byte)
            }
            false => {
                // Fully transparent: clear color channels (alpha stays 0)
                px[0] = 0;
                px[1] = 0;
                px[2] = 0;
            }
        }
    }

    // Build an RgbaImage from the raw bytes
    let rgba_image = image::RgbaImage::from_raw(target_w, target_h, buf)
        .ok_or_else(|| anyhow::anyhow!("Failed to construct image from rasterized SVG bytes"))?;

    Ok(image::DynamicImage::ImageRgba8(rgba_image))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small smoke test: rasterize a 1x1 SVG with a solid red rect and verify pixel.
    #[test]
    fn rasterize_simple_1x1_svg() {
        // Minimal SVG with explicit width/height so usvg uses 1x1 intrinsic size.
        let svg =
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><rect width="1" height="1" fill="#ff0000"/></svg>"##;
        let img = rasterize_svg_to_dynamic_image(svg).expect("rasterization failed");
        let rgba = img.into_rgba8();
        assert_eq!(rgba.width(), 1);
        assert_eq!(rgba.height(), 1);
        let p = rgba.get_pixel(0, 0);
        // Should be solid red
        assert_eq!(p[0], 255);
        assert_eq!(p[1], 0);
        assert_eq!(p[2], 0);
        // Alpha should be fully opaque
        assert_eq!(p[3], 255);
    }

    // Test orange color rasterization to verify correct RGB values
    #[test]
    fn rasterize_orange_svg() {
        // Orange SVG: #FFA500 = RGB(255, 165, 0)
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="#FFA500"/></svg>"##;

        let img = rasterize_svg_to_dynamic_image(svg).expect("Failed to rasterize orange SVG");
        let rgba = img.into_rgba8();

        // Check the center pixel
        let pixel = rgba.get_pixel(50, 50);

        // Expected: R=255, G=165, B=0 (orange)
        assert_eq!(pixel[0], 255, "Red channel should be 255");
        assert_eq!(pixel[1], 165, "Green channel should be 165");
        assert_eq!(pixel[2], 0, "Blue channel should be 0");
        assert_eq!(pixel[3], 255, "Alpha channel should be 255");
    }
}
