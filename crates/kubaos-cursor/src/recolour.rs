//! Recolouring Bibata's SVGs. Its sources draw every cursor in three placeholder colours, which its
//! own build replaces per variant (`render.json`):
//! <https://github.com/ful1e5/Bibata_Cursor/blob/v2.0.7/render.json>
//!
//! - `#00FF00`, the body;
//! - `#0000FF`, the outline;
//! - `#FF0000`, the disc behind the busy cursors' spinner.
//!
//! Every other colour (the spinner's segments, the green plus of `copy`, the red of `no-drop`) is
//! the same in every Bibata variant and stays.

use crate::Colors;

const FILL: &[u8; 6] = b"00FF00";
const OUTLINE: &[u8; 6] = b"0000FF";
const WATCH: &[u8; 6] = b"FF0000";

/// `svg` with its placeholder colours replaced by `colors`, in either case.
pub fn recolour(svg: &str, colors: &Colors) -> String {
    let bytes = svg.as_bytes();
    let mut out = String::with_capacity(svg.len());
    let mut copied = 0;
    let mut i = 0;
    while let Some(offset) = svg[i..].find('#') {
        let at = i + offset;
        let hex = bytes.get(at + 1..at + 7);
        let replacement = hex.and_then(|hex| {
            [(FILL, colors.fill), (OUTLINE, colors.outline), (WATCH, colors.watch)]
                .into_iter()
                .find(|(placeholder, _)| hex.eq_ignore_ascii_case(*placeholder))
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
