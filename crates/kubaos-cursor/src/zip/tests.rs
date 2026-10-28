use super::*;

/// Reads an archive the way libzip does: the end record, then each central entry, then the local
/// header it points at, checking that both agree and that each file's CRC matches.
pub fn read(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let half = |at: usize| usize::from(u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()));
    let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    let end = bytes.len() - 22;
    assert_eq!(word(end), END_OF_CENTRAL);
    let count = half(end + 10);
    let mut at = word(end + 16) as usize;
    assert_eq!(at + word(end + 12) as usize, end);
    let mut files = Vec::new();
    for _ in 0..count {
        assert_eq!(word(at), CENTRAL_HEADER);
        assert_eq!(half(at + 10), 0, "stored");
        let (crc, size, name_len) = (word(at + 16), word(at + 20) as usize, half(at + 28));
        let local = word(at + 42) as usize;
        let name = String::from_utf8(bytes[at + 46..at + 46 + name_len].to_vec()).unwrap();
        assert_eq!(word(local), LOCAL_HEADER);
        assert_eq!(word(local + 14), crc);
        assert_eq!(half(local + 26), name_len);
        let start = local + 30 + name_len + half(local + 28);
        let data = bytes[start..start + size].to_vec();
        assert_eq!(crc32fast::hash(&data), crc);
        files.push((name, data));
        at += 46 + name_len;
    }
    files
}

#[test]
fn files_read_back_by_name_in_order() {
    let mut zip = ZipWriter::new();
    zip.add("meta.hl", b"hotspot_x = 0.5\n");
    zip.add("left_ptr.svg", b"<svg/>");

    let files = read(&zip.finish());

    assert_eq!(
        files,
        [
            ("meta.hl".to_owned(), b"hotspot_x = 0.5\n".to_vec()),
            ("left_ptr.svg".to_owned(), b"<svg/>".to_vec())
        ]
    );
}

#[test]
fn an_empty_archive_is_only_the_end_record() {
    assert_eq!(ZipWriter::new().finish().len(), 22);
}

#[test]
fn the_same_files_make_the_same_bytes() {
    let archive = || {
        let mut zip = ZipWriter::new();
        zip.add("a", b"1");
        zip.finish()
    };

    assert_eq!(archive(), archive());
}
