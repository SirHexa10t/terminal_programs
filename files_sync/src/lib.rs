#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

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
