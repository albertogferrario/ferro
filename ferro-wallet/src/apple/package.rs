//! .pkpass ZIP assembly.
//!
//! Apple Wallet rejects deflated entries — always use `CompressionMethod::Stored`
//! (RESEARCH.md Pitfall 1).

use std::io::{Cursor, Write};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::WalletError;

pub(crate) fn zip_pkpass(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (name, bytes) in entries {
            zip.start_file(name, opts)
                .map_err(|e| WalletError::ApplePackage(format!("start_file {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| WalletError::ApplePackage(format!("write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| WalletError::ApplePackage(format!("finish: {e}")))?;
    }
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::ZipArchive;

    #[test]
    fn zip_pkpass_returns_valid_zip() {
        let entries = vec![
            ("a.txt".to_string(), b"alpha".to_vec()),
            ("b.txt".to_string(), b"beta".to_vec()),
        ];
        let bytes = zip_pkpass(&entries).expect("zip_pkpass");

        // Verify ZIP local-file-header magic.
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04], "ZIP magic bytes");

        // Re-parse and assert exactly 2 entries with expected names.
        let mut archive = ZipArchive::new(Cursor::new(&bytes)).expect("parse zip");
        assert_eq!(archive.len(), 2, "must have exactly 2 entries");

        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"a.txt".to_string()));
        assert!(names.contains(&"b.txt".to_string()));
    }

    #[test]
    fn zip_pkpass_uses_stored_compression() {
        let entries = vec![
            ("a.txt".to_string(), b"alpha".to_vec()),
            ("b.txt".to_string(), b"beta".to_vec()),
        ];
        let bytes = zip_pkpass(&entries).expect("zip_pkpass");

        let mut archive = ZipArchive::new(Cursor::new(&bytes)).expect("parse zip");
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            assert_eq!(
                file.compression(),
                CompressionMethod::Stored,
                "entry {} ({}) must use Stored compression",
                i,
                file.name(),
            );
        }
    }
}
