use serde::{Deserialize, Serialize};

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use base64::Engine as _;
use os_str_bytes::OsStrBytes;



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
    const SEP: &'static str = "  ";

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


    pub fn path_key(&self) -> &str {
        &self.path_key
    }

    pub fn serialize_line(&self) -> String {
        let key_json = serialize_escaped(&self.path_key);
        let rec_json = serialize_escaped(&self.record);
        format!("{key_json}{}{rec_json}", Self::SEP)
    }

    pub fn deserialize_line(line: &str) -> Self {
        let (key_json, rec_json) = line
            .split_once(Self::SEP)
            .expect("invalid tracking line (missing separator)");

        let path_key: String = serde_json::from_str(key_json)
            .unwrap_or_else(|e| panic!("invalid path json: {e}"));

        let record: FileMeta = serde_json::from_str(rec_json)
            .unwrap_or_else(|e| panic!("invalid record json: {e}"));

        Self { path_key, record }
    }

}

pub fn serialize_escaped<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}


fn mtime_ns(md: &fs::Metadata) -> i128 {
    let t = md.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128), // handle pre-epoch if it ever happens
    }
}



