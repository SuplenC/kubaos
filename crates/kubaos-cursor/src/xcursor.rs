//! The XCursor file format: a table of contents, then one image chunk per size and frame, every
//! number a little-endian `u32`. libXcursor picks the nominal size nearest the one asked for, and
//! plays the images of that size in file order.
//! <https://www.x.org/releases/current/doc/man/man3/Xcursor.3.xhtml>

/// One image of a cursor file: one size of a still cursor, or one frame of an animated one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// The size it's drawn for, which libXcursor matches against `XCURSOR_SIZE`.
    pub nominal: u32,
    pub width: u32,
    pub height: u32,
    pub xhot: u32,
    pub yhot: u32,
    /// How long the frame shows, in milliseconds; 0 for a still cursor.
    pub delay_ms: u32,
    /// Premultiplied ARGB, row by row.
    pub pixels: Vec<u32>,
}

const MAGIC: &[u8; 4] = b"Xcur";
const FILE_HEADER: u32 = 16;
const FILE_VERSION: u32 = 0x0001_0000;
const TOC_ENTRY: u32 = 12;
const IMAGE_TYPE: u32 = 0xfffd_0002;
const IMAGE_HEADER: u32 = 36;
const IMAGE_VERSION: u32 = 1;

/// The cursor file holding `images`, in their order.
pub fn encode(images: &[Image]) -> Vec<u8> {
    let count = images.len() as u32;
    let pixel_bytes: usize = images.iter().map(|image| image.pixels.len() * 4).sum();
    let mut out = Vec::with_capacity((FILE_HEADER + count * (TOC_ENTRY + IMAGE_HEADER)) as usize + pixel_bytes);
    out.extend_from_slice(MAGIC);
    for word in [FILE_HEADER, FILE_VERSION, count] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    let mut position = FILE_HEADER + count * TOC_ENTRY;
    for image in images {
        for word in [IMAGE_TYPE, image.nominal, position] {
            out.extend_from_slice(&word.to_le_bytes());
        }
        position += IMAGE_HEADER + image.pixels.len() as u32 * 4;
    }
    for image in images {
        let header = [
            IMAGE_HEADER,
            IMAGE_TYPE,
            image.nominal,
            IMAGE_VERSION,
            image.width,
            image.height,
            image.xhot,
            image.yhot,
            image.delay_ms,
        ];
        for word in header.into_iter().chain(image.pixels.iter().copied()) {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests;
