//! The hyprcursor theme format: `manifest.hl` names the theme and its cursors directory, and each
//! cursor is a `<name>.hlc` there, a ZIP of its `meta.hl` and its images. The images here are the
//! recoloured SVGs, which Hyprland rasterises itself at the size and scale it needs.
//! <https://github.com/hyprwm/hyprcursor/blob/main/docs/MAKING_THEMES.md>
//!
//! # Not here
//!
//! - Writing the files into a theme directory: `theme`.

use std::fmt::Write as _;

use crate::Colors;
use crate::Shape;
use crate::recolour::recolour;
use crate::source::CANVAS;
use crate::zip::ZipWriter;

/// The cursors directory the manifest names.
pub const CURSORS_DIR: &str = "hyprcursors";

/// The theme's `manifest.hl`.
pub fn manifest(name: &str) -> String {
    format!(
        "name = {name}\n\
         description = Bibata Modern in the wallpaper's colours, written by kubaos cursor\n\
         version = 1\n\
         cursors_directory = {CURSORS_DIR}\n"
    )
}

/// A cursor's `meta.hl`: its hotspot as a fraction of the canvas, the names it answers to, and its
/// frames with their delay. An SVG image takes the size 0; hyprcursor ignores it for SVGs.
pub fn meta(shape: &Shape) -> String {
    let (x, y) = shape.hotspot;
    let mut meta = format!(
        "resize_algorithm = bilinear\nhotspot_x = {:.4}\nhotspot_y = {:.4}\n",
        f64::from(x) / f64::from(CANVAS),
        f64::from(y) / f64::from(CANVAS),
    );
    for alias in &shape.aliases {
        let _ = writeln!(meta, "define_override = {alias}");
    }
    let animated = shape.frames.len() > 1;
    for file in frame_files(shape) {
        if animated {
            let _ = writeln!(meta, "define_size = 0, {file}, {}", shape.delay_ms);
        } else {
            let _ = writeln!(meta, "define_size = 0, {file}");
        }
    }
    meta
}

/// The `.hlc` archive of `shape`, its SVGs recoloured with `colors`.
pub fn archive(shape: &Shape, colors: &Colors) -> Vec<u8> {
    let mut zip = ZipWriter::new();
    zip.add("meta.hl", meta(shape).as_bytes());
    for (file, svg) in frame_files(shape).zip(&shape.frames) {
        zip.add(&file, recolour(svg, colors).as_bytes());
    }
    zip.finish()
}

/// The file names of a shape's frames in its archive: `left_ptr.svg`, or `wait-01.svg` and on for
/// an animated one. hyprcursor takes only `[A-Za-z0-9_.-]` in them, as Bibata's names are.
fn frame_files(shape: &Shape) -> impl Iterator<Item = String> + '_ {
    let animated = shape.frames.len() > 1;
    (1..=shape.frames.len()).map(move |i| {
        if animated { format!("{}-{i:02}.svg", shape.name) } else { format!("{}.svg", shape.name) }
    })
}

#[cfg(test)]
mod tests;
