use crate::write_tracking_file;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs as unix_fs;  // we're supporting unix filesystem features such as symlinks


use test_case::test_case;

#[test_case("$HOME/Downloads")]
fn writes_empty_tracking_file_into_dir(dir_spec: &str) {
    let base_dir = expand_home(dir_spec);
    let file_path = write_tracking_file(&base_dir);  // if this doesn't panic, we're good

    let _ = fs::remove_file(&file_path);  // cleanup
}


fn expand_home(s: &str) -> PathBuf {
    if s.starts_with("$HOME/") || s == "$HOME" {
        let home = env::var("HOME").expect("HOME is not set");
        let rest = s.strip_prefix("$HOME").unwrap();
        return PathBuf::from(home).join(rest.trim_start_matches('/'));
    }
    PathBuf::from(s)
}


#[test]
fn tracking_file_is_not_empty_after_mapping_fixture() {
    // Arrange: create the fixture under ./testing (project dir).
    creates_complicated_testing_scenario_in_project_dir();

    // Act: write the tracking file into ./testing.
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tracking_path = write_tracking_file(project_root.join("testing"));

    // Assert: line count is not zero
    let content = std::fs::read_to_string(&tracking_path).unwrap();
    let line_count = content.lines().count();
    assert_ne!(line_count, 0, "tracking file should not be empty");
}


fn creates_complicated_testing_scenario_in_project_dir() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = project_root.join("testing");

    let _ = fs::remove_dir_all(&root);

    create_entry(&project_root, "./testing/f1/a.txt", b"");
    create_entry(&project_root, "./testing/f1/b.txt", b"hello world");

    create_entry(&project_root, "./testing/f2/a.txt", b"another a");
    create_entry(&project_root, "./testing/f2/with space", b"space");
    create_entry(&project_root, "./testing/f2/special!@#$%^&*()-+`\"\'", b"specials");
    create_entry(&project_root, "./testing/f2/with\nnewline", b"newline");

    create_entry(&project_root, "./testing/f3/inner1", b"");
    create_entry(&project_root, "./testing/f3/f4/inner2", b"");

    create_entry(&project_root, "./testing/f2/.hidden", b"hidden");
    create_entry(&project_root, "./testing/f2/with\ttab", b"tab");
    create_entry(&project_root, "./testing/f2/unicode_ハンバーガー_🍣", b"unicode");
    create_entry(&project_root, "./testing/empty_dir/", b"");

    create_entry(&project_root, "./testing/f4/inner2", b"another inner2");

    // Symlinks (created as working relative symlinks)
    // ./testing/f5/sl1 -> ./testing/f1/b.txt
    create_symlink(&project_root, "./testing/f5/sl1", "../f1/b.txt");
    // ./testing/f5/sl2 -> ./testing/f5/sl1
    create_symlink(&project_root, "./testing/f5/sl2", "sl1");
    // ./testing/f5/f6/sl3 -> ./testing/f5/
    create_symlink(&project_root, "./testing/f5/f6/sl3", "../..");

    // Optional: call the function under test now that the fixture exists
    let _ = write_tracking_file(&root);
}

fn create_entry(root: &Path, rel: &str, contents: &[u8]) -> PathBuf {
    let rel = rel.strip_prefix("./").unwrap_or(rel);

    if rel.ends_with('/') {
        let dir_path = root.join(rel.trim_end_matches('/'));
        fs::create_dir_all(&dir_path).unwrap();
        return dir_path;
    }

    let file_path = root.join(rel);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, contents).unwrap();
    file_path
}

fn create_symlink(root: &Path, link_rel: &str, target: &str) -> PathBuf {
    let link_rel = link_rel.strip_prefix("./").unwrap_or(link_rel);
    let link_path = root.join(link_rel);

    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    // Replace if it already exists (file/dir/symlink)
    let _ = fs::remove_file(&link_path);
    let _ = fs::remove_dir_all(&link_path);

    unix_fs::symlink(target, &link_path).unwrap();
    link_path
}
