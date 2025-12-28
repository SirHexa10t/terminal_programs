#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::io::{Write, BufWriter};

pub const TRACKING_FILENAME: &str = "filesync_tracking.txt";

pub fn write_tracking_file(dir: impl AsRef<Path>) -> PathBuf {
    let dir = dir.as_ref();

    match fs::metadata(dir) {
        Ok(md) if md.is_dir() => {}
        Ok(_) => panic!("not a directory: '{}'", dir.display()),
        Err(e) => panic!("metadata failed for '{}': {}", dir.display(), e),
    }

    let file_path = dir.join(TRACKING_FILENAME);

    match fs::symlink_metadata(&file_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}  // doesn't exist: ok
        Ok(m) if m.is_file() => {}                             // exists and is file: ok
        Ok(_) => panic!("tracking path exists but is not a file: '{}'", file_path.display()),
        Err(e) => panic!("metadata failed for '{}': {}", file_path.display(), e),
    }

    if let Err(e) = fs::File::create(&file_path) {
        panic!("failed to create '{}': {}", file_path.display(), e);
    }

    file_path
}


fn list_tree_paths(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())          // ignore traversal errors for now
        .filter(|e| e.depth() != 0)      // exclude root itself
        .map(|e| e.path().strip_prefix(dir).unwrap().to_path_buf())
        .collect();

    // deterministic ordering (Linux): compare raw bytes of the OsStr
    out.sort();
    out
}

pub fn write_tracking_file_with_listing(dir: impl AsRef<Path>) -> PathBuf {
    let dir = dir.as_ref();
    let tracking_path = write_tracking_file(dir);

    // build listing
    let mut paths = list_tree_paths(dir);
    paths.retain(|p| p.as_os_str() != TRACKING_FILENAME);  // exclude the tracking file itself

    // write one relative path per line (lossy display; fine for now)
    let file = fs::File::create(&tracking_path)
        .unwrap_or_else(|e| panic!("failed to create '{}': {}", tracking_path.display(), e));
    let mut w = BufWriter::new(file);

    fn escape_tracking(s: &str) -> String {
        s.chars().flat_map(|c| c.escape_default()).collect()
    }
    
    for p in paths {
        let escaped = escape_tracking(&p.to_string_lossy());
        writeln!(w, "{}", escaped)
            .unwrap_or_else(|e| panic!("failed to write to '{}': {}", tracking_path.display(), e));
    }

    tracking_path
}
