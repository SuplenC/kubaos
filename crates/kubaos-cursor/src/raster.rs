//! Rasterising a recoloured SVG into XCursor pixels with resvg, Bibata's drop shadows (SVG
//! filters) included.

use resvg::tiny_skia;
use resvg::usvg;

/// A parsed SVG, drawn at as many sizes as needed.
#[derive(Debug)]
pub struct Drawing(usvg::Tree);

/// An SVG resvg couldn't parse.
#[derive(Debug, thiserror::Error)]
#[error("parsing a cursor SVG")]
pub struct RasterError(#[source] usvg::Error);

impl Drawing {
    /// Parses `svg`.
    ///
    /// # Errors
    ///
    /// [`RasterError`] when it isn't an SVG resvg reads.
    pub fn parse(svg: &str) -> Result<Self, RasterError> {
        usvg::Tree::from_str(svg, &usvg::Options::default()).map(Self).map_err(RasterError)
    }

    /// The whole canvas drawn into `px` by `px` pixels, as premultiplied ARGB, row by row: the
    /// layout of an XCursor image.
    pub fn render(&self, px: u32) -> Vec<u32> {
        let Some(mut pixmap) = tiny_skia::Pixmap::new(px, px) else {
            return Vec::new();
        };
        let size = self.0.size();
        let transform = tiny_skia::Transform::from_scale(px as f32 / size.width(), px as f32 / size.height());
        resvg::render(&self.0, transform, &mut pixmap.as_mut());
        pixmap
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&[r, g, b, a]| u32::from_be_bytes([a, r, g, b]))
            .collect()
    }
}

#[cfg(test)]
mod tests;
