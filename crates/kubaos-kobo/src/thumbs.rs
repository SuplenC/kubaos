//! Thumbnails of the payload's wallpapers, for the Wallpapers screen.
//!
//! # Owns
//!
//! Finding each wallpaper's thumbnail and decoding it: the payload's small JPEG, else the
//! original shrunk to [`WIDTH`], else nothing (the screen draws a placeholder). [`cropped`] cuts
//! one to [`SHAPE`], so the cards they sit on are all one height.
//!
//! # Not here
//!
//! Making the payload's thumbnails (`install/image/lib/assemble.sh`), running the decode off the
//! UI thread (`app.rs`, `load_thumbs`), drawing them (`view.rs`, `wallpapers`).

use std::path::Path;
use std::path::PathBuf;

use image::RgbaImage;
use kubaos_core::probe::WallpaperFile;
use kubaos_core::probe::WallpaperSet;

use crate::model;

/// The width of a thumbnail: the payload's are made at it (`assemble.sh`, `THUMB_WIDTH`), and an
/// original is shrunk to it.
pub const WIDTH: u32 = 480;

/// The shape every thumbnail is cropped to, as a width and a height in pixels: a screen's 16:10.
pub const SHAPE: (u32, u32) = (16, 10);

/// Where an installed system keeps the originals. The live ISO has none: they're in the
/// squashfs, which isn't mounted before the install.
pub const ORIGINALS: &str = "/usr/share/kubaos/wallpapers";

/// The set's thumbnails that could be found and decoded, keyed like [`model::wallpaper_key`].
pub fn decode(payload: &Path, originals: &Path, set: &WallpaperSet) -> Vec<(String, RgbaImage)> {
    set.files
        .iter()
        .filter_map(|f| {
            let img = candidates(payload, originals, set, f)
                .into_iter()
                .find_map(|p| image::open(p).ok())?;
            let img = if img.width() > WIDTH {
                img.thumbnail(WIDTH, u32::MAX)
            } else {
                img
            };
            Some((model::wallpaper_key(set, f), img.to_rgba8()))
        })
        .collect()
}

/// `img` cut about its middle to [`SHAPE`]: the wider side trimmed evenly at both ends.
pub fn cropped(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let (sw, sh) = SHAPE;
    let (width, height) = if u64::from(w) * u64::from(sh) > u64::from(h) * u64::from(sw) {
        (h * sw / sh, h)
    } else {
        (w, w * sh / sw)
    };
    image::imageops::crop_imm(img, (w - width) / 2, (h - height) / 2, width.max(1), height.max(1)).to_image()
}

/// Where a wallpaper's picture may be, best first: its thumbnail, then the original (a video's
/// poster, `<name>.poster.jpg` next to it).
fn candidates(payload: &Path, originals: &Path, set: &WallpaperSet, f: &WallpaperFile) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(thumb) = &f.thumb {
        v.push(payload.join(thumb));
    }
    let original = originals.join(&set.id).join(&f.name);
    if f.is_video() {
        v.push(original.with_extension("poster.jpg"));
    } else {
        v.push(original);
    }
    v
}

#[cfg(test)]
mod tests;
