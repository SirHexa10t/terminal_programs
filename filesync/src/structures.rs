use serde::{Deserialize, Serialize};

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use base64::Engine as _;
use os_str_bytes::OsStrBytes;
use rayon::prelude::*;
use unicode_width::UnicodeWidthStr;


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    // lossless path bytes, base64 (relative path bytes as seen by the OS)
    pub path_b64: String,

    // file/dir/symlink/other
    pub ty: NodeType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    // Store times as ns since UNIX epoch (portable, sortable).
    pub mtime_ns: i128,

    // Optional (Linux): permissions bits (e.g., 0o644). Omit on platforms where it’s awkward.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,

    // Only present when ty == Symlink.
    // Store as JSON string (UTF-8). If you later need lossless non-UTF8 targets on Unix,
    // add link_target_b64 as a parallel field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_target: Option<String>,
}


#[derive(Debug, Clone)]
pub struct ManifestEntry {
    /// Relative path used as the primary sort key (human-readable, UTF-8-ish).
    path_key: String,
    record: FileMeta,
}

impl ManifestEntry {

    pub fn from_rel_path(root: &Path, rel: PathBuf) -> Self {
        let full = root.join(&rel);

        let md = fs::symlink_metadata(&full)
            .unwrap_or_else(|e| panic!("symlink_metadata failed for '{}': {e}", full.display()));

        let ft = md.file_type();

        let ty = match (ft.is_dir(), ft.is_file(), ft.is_symlink()) {
            (true, _, _) => NodeType::Dir,
            (_, true, _) => NodeType::File,
            (_, _, true) => NodeType::Symlink,
            _ => NodeType::Other,
        };

        let size = ft.is_file().then(|| md.len());

        fn lossy_utf8(p: &std::path::Path) -> String { p.to_string_lossy().into_owned() }

        let link_target = ft.is_symlink().then(|| {
            lossy_utf8(
                &fs::read_link(&full)
                    .unwrap_or_else(|e| panic!("read_link failed for '{}': {e}", full.display()))
            )
        });

        #[cfg(unix)]
        let mode = Some(md.mode() & 0o7777);
        #[cfg(not(unix))]
        let mode = None;

        ManifestEntry {
            path_key: lossy_utf8(&rel),
            record: FileMeta {
                path_b64: base64::engine::general_purpose::STANDARD_NO_PAD.encode(&*rel.to_raw_bytes()),
                ty,
                size,
                mtime_ns: mtime_ns(&md),
                mode,
                link_target,
            },
        }
    }

    pub fn deserialize_line(line: &str) -> Self {
        let mut de = serde_json::Deserializer::from_str(line);

        let path_key = String::deserialize(&mut de)
            .unwrap_or_else(|e| panic!("invalid path json: {e}; line={line:?}"));

        // record follows after whitespace
        let record = FileMeta::deserialize(&mut de)
            .unwrap_or_else(|e| panic!("invalid record json: {e}; line={line:?}"));

        // Verify there is nothing else after the record (besides whitespace)
        if !de.end().is_ok() {
            panic!("tracking line has trailing junk; line={line:?}");
        }

        ManifestEntry { path_key, record }
    }


    pub fn path_key(&self) -> &str {
        &self.path_key
    }

    fn serialize(&self) -> (String, String) {
        (serde_json::to_string(&self.path_key).unwrap(), serde_json::to_string(&self.record).unwrap())
    }

    /// Deserialize only the leading JSON string (path_key) from a line (even if there's nothing after)
    pub fn deserialize_path_key(line: &str) -> String {
        let mut it = serde_json::Deserializer::from_str(line).into_iter::<String>();

        it.next()
            .unwrap_or_else(|| panic!("tracking line missing path key; line={line:?}"))
            .unwrap_or_else(|e| panic!("invalid path json: {e}; line={line:?}"))
    }

    /// Render entries as aligned lines: `<path_key_json><spaces><record_json>\n`
    /// where `record_json` starts at the same column for all lines.
    pub fn serialize_manifests(entries: &[ManifestEntry]) -> String {
        // Parallel map: ManifestEntry -> (key, record)
        let pairs: Vec<(String, String)> = entries.par_iter()
            .map(ManifestEntry::serialize)
            .collect();

        let pad_to = pairs.par_iter()
            .map(|(k, _)| UnicodeWidthStr::width(k.as_str()))  // align visually, by width of characters, not byte-length
            .max()
            .unwrap_or(0)
            + 2;  // minimum 2 spaces in-between

        // Build final string (sequential join; avoids contention)
        let mut lines: Vec<String> = pairs.into_par_iter()
            .map(|(k, r)| [
                k.as_str(),
                &" ".repeat(pad_to.saturating_sub(UnicodeWidthStr::width(k.as_str()))),
                r.as_str()
            ].into_iter().collect())
            .collect();

        lines.par_sort_unstable();
        lines.join("\n")
    }


    pub fn deserialize_manifests(content: &str) -> Vec<ManifestEntry> {
        let mut entries: Vec<ManifestEntry> = content.par_lines()
            .filter(|l| !l.is_empty())
            .map(ManifestEntry::deserialize_line)
            .collect();

        entries.par_sort_unstable_by(|a, b| a.path_key().cmp(b.path_key()));
        entries
    }
}



fn mtime_ns(md: &fs::Metadata) -> i128 {
    let t = md.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128), // handle pre-epoch if it ever happens
    }
}



