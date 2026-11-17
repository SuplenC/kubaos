//! The size of an app's PNG or JPEG icon, read from the file's own header.

/// A picture's width and height in pixels, from a PNG's `IHDR` or a JPEG's start-of-frame
/// marker; none for bytes that are neither.
pub fn size(bytes: &[u8]) -> Option<(u32, u32)> {
    png_size(bytes).or_else(|| jpeg_size(bytes))
}

/// A PNG's size: its signature, then the `IHDR` chunk, which the format puts first.
/// <https://www.w3.org/TR/png-3/#11IHDR>
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    let rest = bytes.strip_prefix(SIGNATURE)?;
    if rest.get(4..8)? != b"IHDR" {
        return None;
    }
    let word = |at: usize| Some(u32::from_be_bytes(rest.get(at..at + 4)?.try_into().ok()?));
    Some((word(8)?, word(12)?))
}

/// A JPEG's size: the first start-of-frame marker's (`SOF0`–`SOF15`, but not `DHT`, `JPG` or
/// `DAC`, which share the range), walking the segments from the start-of-image marker.
/// <https://www.w3.org/Graphics/JPEG/itu-t81.pdf>, B.2.2
fn jpeg_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut at = 2;
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    loop {
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let marker = *bytes.get(at + 1)?;
        // Fill bytes before a marker.
        if marker == 0xFF {
            at += 1;
            continue;
        }
        let length = usize::from(u16::from_be_bytes([*bytes.get(at + 2)?, *bytes.get(at + 3)?]));
        if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            let height = u16::from_be_bytes([*bytes.get(at + 5)?, *bytes.get(at + 6)?]);
            let width = u16::from_be_bytes([*bytes.get(at + 7)?, *bytes.get(at + 8)?]);
            return Some((u32::from(width), u32::from(height)));
        }
        at += 2 + length;
    }
}

/// The comment a PNG or a JPEG carries, as a picture's licence or source is written in it: a PNG's
/// first `tEXt` chunk with the keyword `Comment` (PNG 11.3.4.3), or a JPEG's first `COM` segment.
#[cfg(test)]
pub fn comment(bytes: &[u8]) -> Option<String> {
    let text = |body: &[u8]| String::from_utf8_lossy(body).into_owned();
    if let Some(mut rest) = bytes.strip_prefix(b"\x89PNG\r\n\x1a\n".as_slice()) {
        while rest.len() >= 12 {
            let length = usize::try_from(u32::from_be_bytes(rest[..4].try_into().ok()?)).ok()?;
            let body = rest.get(8..8 + length)?;
            if &rest[4..8] == b"tEXt"
                && let Some(value) = body.strip_prefix(b"Comment\0".as_slice())
            {
                return Some(text(value));
            }
            rest = rest.get(12 + length..)?;
        }
        return None;
    }
    let mut at = 2;
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    loop {
        let marker = *bytes.get(at + 1)?;
        let length = usize::from(u16::from_be_bytes([*bytes.get(at + 2)?, *bytes.get(at + 3)?]));
        if marker == 0xFE {
            return Some(text(bytes.get(at + 4..at + 2 + length)?));
        }
        // The image data starts: no comment comes before it.
        if marker == 0xDA {
            return None;
        }
        at += 2 + length;
    }
}

/// The start of a PNG of `width` × `height`: its signature and `IHDR`, all `size` reads, then
/// bytes that aren't UTF-8, as a picture's aren't.
#[cfg(test)]
pub fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    // Bit depth, colour type, compression, filter and interlace, then a CRC no reader here checks.
    bytes.extend([8, 6, 0, 0, 0, 0xFF, 0xFE, 0x80, 0x00]);
    bytes
}

/// The start of a JPEG of `width` × `height`: its start-of-image marker, an `APP0` segment
/// before the frame, as a JFIF file has, then its baseline start-of-frame.
#[cfg(test)]
pub fn jpeg(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x4A, 0x46, 0xFF, 0xC0, 0x00, 0x11, 0x08,
    ];
    bytes.extend(height.to_be_bytes());
    bytes.extend(width.to_be_bytes());
    bytes.extend([0x03, 0x01, 0x22, 0x00, 0xFF, 0xD9]);
    bytes
}

#[cfg(test)]
mod tests {
    use super::comment;
    use super::jpeg;
    use super::png;
    use super::size;

    #[test]
    fn a_png_and_a_jpeg_say_their_size_and_other_bytes_none() {
        assert_eq!(size(&png(128, 64)), Some((128, 64)));
        assert_eq!(size(&jpeg(300, 200)), Some((300, 200)));
        assert_eq!(size(b"<svg/>"), None);
        assert_eq!(size(&png(1, 1)[..20]), None);
    }

    #[test]
    fn a_pngs_comment_chunk_and_a_jpegs_comment_segment_are_read() {
        let mut with_comment = png(16, 16);
        with_comment.extend(17u32.to_be_bytes());
        with_comment.extend(b"tEXtComment\0MIT, ours");
        with_comment.extend([0; 4]);
        let mut jpeg_comment = vec![0xFF, 0xD8, 0xFF, 0xFE, 0x00, 0x06];
        jpeg_comment.extend(b"MIT!");

        assert_eq!(comment(&with_comment).as_deref(), Some("MIT, ours"));
        assert_eq!(comment(&jpeg_comment).as_deref(), Some("MIT!"));
        assert_eq!(comment(&png(16, 16)), None);
    }
}
