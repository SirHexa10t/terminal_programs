use crate::{write_tracking_file, write_tracking_file_with_listing, TRACKING_FILENAME};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs as unix_fs;  // we're supporting unix filesystem features such as symlinks
use std::process::Command;
use rayon::prelude::*;


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
fn tracking_file_compare_with_external_command() {
    let dir_a = creates_complicated_testing_tree("A");
    let dir_a_tracker = write_tracking_file_with_listing(&dir_a);

    let tracker_content = path_keys_from_tracking_file(&dir_a_tracker);
    let baseline_out = find_escaped_output(&dir_a);

    assert_eq!(tracker_content, baseline_out,
               "tracking file line count mismatch: tracking={}, baseline={}", tracker_content.join("\n"), baseline_out.join("\n"));
}


fn creates_complicated_testing_tree(subdir: &str) -> PathBuf {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = project_root.join("testing").join(subdir);

    let _ = fs::remove_dir_all(&root);

    // simple
    create_entry(&root, "f1/a.txt", b"");
    create_entry(&root, "f1/b.txt", b"hello world");

    // problematic chars
    create_entry(&root, "f2/a.txt", b"another a");
    create_entry(&root, "f2/ with space", b"space");
    create_entry(&root, "f2/special!@#$%^&*()-+`\"\'", b"specials");
    create_entry(&root, "f2/with\nnewline", b"newline");
    create_entry(&root, "f2/with\ttab", b"tab");
    create_entry(&root, "f2/unicode_ハンバーガー_🍣", b"unicode");

    // hierarchy/hidden
    create_entry(&root, "f3/inner1", b"");
    create_entry(&root, "f3/f4/inner2", b"inner2");
    create_entry(&root, "f3/.hidden", b"hidden");

    // empty
    create_entry(&root, "empty_dir/", b"");

    // extra instances
    create_entry(&root, "f4/inner2", b"another inner2");  // "duplicate" file
    create_entry(&root, &format!("f4/{}", TRACKING_FILENAME), b"another inner2");  // "duplicate" file

    // links
    create_symlink(&root, "f5/sl1", "../f1/b.txt");
    create_symlink(&root, "f5/sl2", "sl1");  // f5/sl2 -> f5/sl1
    create_symlink(&root, "f5/f6/sl3", "../..");  // f5/f6/sl3 -> f5/
    create_symlink(&root, "f5/f6/sl4", expand_home("$HOME/Downloads").to_str().unwrap());  // link outside of project

    root
}

fn create_entry(root: &Path, rel: &str, contents: &[u8]) -> PathBuf {
    // Expect paths relative to `root` (e.g., "f1/a.txt" or "empty_dir/")
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



/// Runs:
///   find . -mindepth 1 -printf '%P\0'
/// Then escapes the output and sorts it.
///
/// Notes:
/// - Uses `sh -lc` because of the pipe.
fn find_escaped_output(dir: &std::path::Path) -> Vec<String> {
    // run "find" command
    let out = Command::new("sh")
        .arg("-lc")
        .arg(r"find . -mindepth 1 -printf '%P\0'")
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run shell command in '{}': {}", dir.display(), e));

    // check that return code was 0, and process/collect the data
    let mut lines: Vec<String> = match out.status.success() {
        false => panic!("find failed in '{}': exit={:?}, stderr={}", dir.display(), out.status.code(), String::from_utf8_lossy(&out.stderr)),
        true => out.stdout.par_split(|&b| b == 0)  // split stdout on \0
            .filter(|s| !s.is_empty())  // sanitize
            .filter(|s| *s != TRACKING_FILENAME.as_bytes())  // sanitize
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect()
    };

    lines.par_sort_unstable();  // no need to preserve equals' order; run it a bit faster
    lines
}

// Tracking file -> sorted Vec<String> of path_keys
fn path_keys_from_tracking_file(tracking_file: &std::path::Path) -> Vec<String> {
    let mut content: Vec<String> = std::fs::read_to_string(tracking_file)
        .unwrap_or_else(|e| panic!("failed to read '{}': {}", tracking_file.display(), e))
        .par_lines()
        .filter(|l| !l.is_empty())
        .map(|line| crate::ManifestEntry::deserialize_line(line).path_key().to_owned())
        .collect();

    content.par_sort_unstable();
    content
}

