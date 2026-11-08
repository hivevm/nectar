//! Packs an EPUB container: a ZIP archive under the rules of EPUB 3.3, sections 4.3.2 and 4.3.3.
//! Nothing in it depends on the time or the host, so the same entries give the same bytes.

use std::collections::BTreeMap;

const MIMETYPE: &[u8] = b"application/epub+zip";
const LOCAL_HEADER: u32 = 0x0403_4b50;
const CENTRAL_HEADER: u32 = 0x0201_4b50;
const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
/// The names of the entries are UTF-8.
const FLAG_UTF8: u16 = 1 << 11;
const STORED: u16 = 0;
const DEFLATE: u16 = 8;
/// 1980-01-01, the earliest date ZIP can hold; the time of day is 00:00:00.
const DOS_DATE: u16 = (1 << 5) | 1;
const DEFLATE_LEVEL: u8 = 6;
/// A count of entries and a size must stay below these: a reader takes them as marks of ZIP64.
const MAX_COUNT: usize = 0xFFFF;
const MAX_SIZE: usize = 0xFFFF_FFFF;

struct Entry {
    method: u16,
    crc: u32,
    size: u32,
    packed: Vec<u8>,
}

/// The archive of `entries`, by their names and in their order after `mimetype`, or what ZIP
/// without ZIP64 cannot hold.
pub(crate) fn pack(entries: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, String> {
    let mut packed = vec![(
        "mimetype",
        Entry {
            method: STORED,
            crc: crc32fast::hash(MIMETYPE),
            size: MIMETYPE.len() as u32,
            packed: MIMETYPE.to_vec(),
        },
    )];
    for (name, bytes) in entries {
        let size =
            below_marks(bytes.len()).ok_or_else(|| format!("{name} is too large for ZIP"))?;
        packed.push((
            name,
            Entry {
                method: DEFLATE,
                crc: crc32fast::hash(bytes),
                size,
                packed: miniz_oxide::deflate::compress_to_vec(bytes, DEFLATE_LEVEL),
            },
        ));
    }
    if packed.len() >= MAX_COUNT {
        return Err("too many entries for ZIP".to_string());
    }
    let count = packed.len() as u16;
    let too_large = || "the archive is too large for ZIP".to_string();

    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, entry) in &packed {
        let offset = below_marks(out.len()).ok_or_else(too_large)?;
        let needed: u16 = if entry.method == STORED { 10 } else { 20 };
        let name_length =
            u16::try_from(name.len()).map_err(|_| format!("the name {name} is too long"))?;
        let packed_size = below_marks(entry.packed.len())
            .ok_or_else(|| format!("{name} is too large for ZIP"))?;

        let common = |bytes: &mut Vec<u8>| {
            put16(bytes, needed);
            put16(bytes, FLAG_UTF8);
            put16(bytes, entry.method);
            put16(bytes, 0);
            put16(bytes, DOS_DATE);
            put32(bytes, entry.crc);
            put32(bytes, packed_size);
            put32(bytes, entry.size);
            put16(bytes, name_length);
            // No extra field.
            put16(bytes, 0);
        };

        put32(&mut out, LOCAL_HEADER);
        common(&mut out);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&entry.packed);

        put32(&mut central, CENTRAL_HEADER);
        // The version that made the entry, then the fields of the local header.
        put16(&mut central, 20);
        common(&mut central);
        // No comment, on the first disk, with no attributes.
        put16(&mut central, 0);
        put16(&mut central, 0);
        put16(&mut central, 0);
        put32(&mut central, 0);
        put32(&mut central, offset);
        central.extend_from_slice(name.as_bytes());
    }

    let offset = below_marks(out.len()).ok_or_else(too_large)?;
    let size = below_marks(central.len()).ok_or_else(too_large)?;
    out.extend_from_slice(&central);
    put32(&mut out, END_OF_CENTRAL_DIRECTORY);
    put16(&mut out, 0);
    put16(&mut out, 0);
    put16(&mut out, count);
    put16(&mut out, count);
    put32(&mut out, size);
    put32(&mut out, offset);
    put16(&mut out, 0);
    Ok(out)
}

fn below_marks(size: usize) -> Option<u32> {
    (size < MAX_SIZE).then_some(size as u32)
}

fn put16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: ADR-0021
    #[test]
    fn a_count_a_reader_takes_for_zip64_is_refused() {
        let entries = |count: usize| -> BTreeMap<String, Vec<u8>> {
            (0..count).map(|i| (format!("{i}"), Vec::new())).collect()
        };
        // With `mimetype`, these are 65534 and 65535 entries.
        assert!(pack(&entries(65533)).is_ok());
        assert_eq!(
            pack(&entries(65534)),
            Err("too many entries for ZIP".to_string())
        );
    }

    #[test]
    fn a_size_a_reader_takes_for_zip64_is_refused() {
        assert_eq!(below_marks(0xFFFF_FFFE), Some(0xFFFF_FFFE));
        assert_eq!(below_marks(0xFFFF_FFFF), None);
    }
}
