//! A ZIP archive of stored (uncompressed) files, the container of a hyprcursor `.hlc` shape, which
//! libhyprcursor opens with libzip. Stored keeps writing a theme a memcpy and a CRC per file; the
//! SVGs are a few kilobytes each.
//! <https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT>

const LOCAL_HEADER: u32 = 0x0403_4b50;
const CENTRAL_HEADER: u32 = 0x0201_4b50;
const END_OF_CENTRAL: u32 = 0x0605_4b50;
/// 1.0, enough for stored files.
const VERSION_NEEDED: u16 = 10;
/// Unix (3) in the high byte, spec version 3.0 (0x1e) in the low one, so the external attributes
/// below are read as a mode.
const VERSION_MADE_BY: u16 = 0x031e;
/// 1980-01-01, the earliest DOS date (month 1 at bit 5, day 1): the archive is the same bytes for
/// the same files.
const DOS_DATE: u16 = 0x0021;
/// A regular file, mode 0644.
const EXTERNAL_ATTRIBUTES: u32 = 0o100_644 << 16;

/// An archive being written.
#[derive(Debug, Default)]
pub(crate) struct ZipWriter {
    out: Vec<u8>,
    central: Vec<u8>,
    count: u16,
}

impl ZipWriter {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Adds `data` as the file `name`.
    pub(crate) fn add(&mut self, name: &str, data: &[u8]) {
        let crc = crc32fast::hash(data);
        let offset = self.out.len() as u32;
        let size = data.len() as u32;
        let name_len = name.len() as u16;

        put32(&mut self.out, LOCAL_HEADER);
        for half in [VERSION_NEEDED, 0, 0, 0, DOS_DATE] {
            put16(&mut self.out, half);
        }
        for word in [crc, size, size] {
            put32(&mut self.out, word);
        }
        put16(&mut self.out, name_len);
        put16(&mut self.out, 0);
        self.out.extend_from_slice(name.as_bytes());
        self.out.extend_from_slice(data);

        put32(&mut self.central, CENTRAL_HEADER);
        for half in [VERSION_MADE_BY, VERSION_NEEDED, 0, 0, 0, DOS_DATE] {
            put16(&mut self.central, half);
        }
        for word in [crc, size, size] {
            put32(&mut self.central, word);
        }
        for half in [name_len, 0, 0, 0, 0] {
            put16(&mut self.central, half);
        }
        put32(&mut self.central, EXTERNAL_ATTRIBUTES);
        put32(&mut self.central, offset);
        self.central.extend_from_slice(name.as_bytes());
        self.count += 1;
    }

    /// The finished archive.
    pub(crate) fn finish(mut self) -> Vec<u8> {
        let central_offset = self.out.len() as u32;
        let central_size = self.central.len() as u32;
        self.out.append(&mut self.central);
        put32(&mut self.out, END_OF_CENTRAL);
        for half in [0, 0, self.count, self.count] {
            put16(&mut self.out, half);
        }
        put32(&mut self.out, central_size);
        put32(&mut self.out, central_offset);
        put16(&mut self.out, 0);
        self.out
    }
}

fn put16(out: &mut Vec<u8>, half: u16) {
    out.extend_from_slice(&half.to_le_bytes());
}

fn put32(out: &mut Vec<u8>, word: u32) {
    out.extend_from_slice(&word.to_le_bytes());
}

#[cfg(test)]
pub(crate) mod tests;
