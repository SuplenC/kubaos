//! Recolouring Bibata's SVGs. Its sources draw every cursor in three placeholder colours, which its
//! own build replaces per variant (`render.json`):
//! <https://github.com/ful1e5/Bibata_Cursor/blob/v2.0.7/render.json>
//!
//! - `#00FF00`, the body;
//! - `#0000FF`, the outline;
//! - `#FF0000`, the disc behind the busy cursors' spinner.
//!
//! The spinner's four segments are fixed colours in every Bibata variant, a rainbow that would
//! clash with any wallpaper, so they're replaced too, each by a tone of the style's spinner. So are
//! the four diagonal resize cursors, which Bibata draws in lighter shades of the same four. The
//! green plus of `copy` and the red of `no-drop` carry meaning and stay.

use crate::Colors;
use crate::Rgb;

const FILL: &[u8; 6] = b"00FF00";
const OUTLINE: &[u8; 6] = b"0000FF";
const WATCH: &[u8; 6] = b"FF0000";
/// The spinner's segments clockwise from the top right (orange, yellow, green, blue), as `wait`
/// and `left_ptr_watch` draw them.
const SPINNER: [&[u8; 6]; 4] = [b"F05024", b"FCB813", b"7EBA41", b"32A0DA"];
/// The corner cursors' shades of the same four, in the same order.
const CORNERS: [&[u8; 6]; 4] = [b"F1613A", b"FDBE2A", b"96C865", b"4FADDF"];

/// `svg` with its placeholder and spinner colours replaced by `colors`, in either case.
pub fn recolour(svg: &str, colors: &Colors) -> String {
    let mut table: Vec<(&[u8; 6], Rgb)> = vec![(FILL, colors.fill), (OUTLINE, colors.outline), (WATCH, colors.watch)];
    for (i, tone) in colors.spinner.into_iter().enumerate() {
        table.push((SPINNER[i], tone));
        table.push((CORNERS[i], tone));
    }
    let bytes = svg.as_bytes();
    let mut out = String::with_capacity(svg.len());
    let mut copied = 0;
    let mut i = 0;
    while let Some(offset) = svg[i..].find('#') {
        let at = i + offset;
        let hex = bytes.get(at + 1..at + 7);
        let replacement = hex.and_then(|hex| {
            table
                .iter()
                .find(|(colour, _)| hex.eq_ignore_ascii_case(*colour))
                .map(|(_, rgb)| rgb)
        });
        match replacement {
            Some(rgb) => {
                out.push_str(&svg[copied..at]);
                out.push_str(&rgb.to_string());
                i = at + 7;
                copied = i;
            }
            None => i = at + 1,
        }
    }
    out.push_str(&svg[copied..]);
    out
}

#[cfg(test)]
mod tests;
