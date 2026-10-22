use super::*;

/// Reads a cursor file the way libXcursor does: the header, the table of contents, then each
/// chunk at its position, checking that the chunk repeats its entry's type and size.
pub fn decode(bytes: &[u8]) -> Vec<Image> {
    let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    assert_eq!(&bytes[..4], MAGIC);
    assert_eq!(word(4), FILE_HEADER);
    assert_eq!(word(8), FILE_VERSION);
    let count = word(12) as usize;
    (0..count)
        .map(|i| {
            let entry = FILE_HEADER as usize + i * TOC_ENTRY as usize;
            let (kind, nominal, position) = (word(entry), word(entry + 4), word(entry + 8) as usize);
            assert_eq!(kind, IMAGE_TYPE);
            assert_eq!(word(position), IMAGE_HEADER);
            assert_eq!(word(position + 4), kind);
            assert_eq!(word(position + 8), nominal);
            assert_eq!(word(position + 12), IMAGE_VERSION);
            let (width, height) = (word(position + 16), word(position + 20));
            let start = position + IMAGE_HEADER as usize;
            let pixels = (0..(width * height) as usize).map(|p| word(start + p * 4)).collect();
            Image {
                nominal,
                width,
                height,
                xhot: word(position + 24),
                yhot: word(position + 28),
                delay_ms: word(position + 32),
                pixels,
            }
        })
        .collect()
}

fn image(nominal: u32, delay_ms: u32, pixel: u32) -> Image {
    Image {
        nominal,
        width: nominal,
        height: nominal,
        xhot: nominal / 4,
        yhot: 1,
        delay_ms,
        pixels: vec![pixel; (nominal * nominal) as usize],
    }
}

#[test]
fn a_still_cursor_in_two_sizes_reads_back_the_same() {
    let images = [image(24, 0, 0xff10_2030), image(48, 0, 0x8008_1018)];

    assert_eq!(decode(&encode(&images)), images);
}

#[test]
fn an_animated_cursors_frames_keep_their_order_and_delay() {
    let images = [image(24, 40, 1), image(24, 40, 2), image(24, 40, 3)];

    let read = decode(&encode(&images));

    assert_eq!(read.iter().map(|i| i.pixels[0]).collect::<Vec<_>>(), [1, 2, 3]);
    assert!(read.iter().all(|i| i.delay_ms == 40));
}

#[test]
fn the_file_is_exactly_its_headers_and_pixels() {
    let images = [image(24, 0, 0), image(32, 0, 0)];

    let bytes = encode(&images);

    assert_eq!(bytes.len(), 16 + 2 * 12 + 2 * 36 + (24 * 24 + 32 * 32) * 4);
}
