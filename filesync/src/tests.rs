use crate::{write_tracking_file, write_tracking_file_with_listing, TRACKING_FILENAME};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs as unix_fs;  // we're supporting unix filesystem features such as symlinks
use std::process::Command;


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
fn tracking_file_contains_the_right_amount_of_entries() {
    creates_complicated_testing_scenario_in_project_dir("testA");

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let testing_dir = project_root.join("testing");
    let tracking_path = write_tracking_file_with_listing(&testing_dir);

    // Count non-empty lines in the external baseline output.
    let content = std::fs::read_to_string(&tracking_path).unwrap();
    let baseline_out = find_escaped_output(&testing_dir);

    fn filtered_line_count(s: &str) -> usize {
        s.lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .filter(|l| !l.contains(TRACKING_FILENAME))
            .count()
    }

    // Count non-empty lines in the external baseline output.
    let line_count = filtered_line_count(&content);
    let baseline_count = filtered_line_count(&baseline_out);

    assert_eq!(line_count, baseline_count,
               "tracking file line count mismatch: tracking={}, baseline={}", line_count, baseline_count);
}


fn creates_complicated_testing_scenario_in_project_dir(subdir: &str) -> PathBuf {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = project_root.join("testing").join(subdir);

    let _ = fs::remove_dir_all(&root);

    create_entry(&root, "f1/a.txt", b"");
    create_entry(&root, "f1/b.txt", b"hello world");

    create_entry(&root, "f2/a.txt", b"another a");
    create_entry(&root, "f2/ with space", b"space");
    create_entry(&root, "f2/special!@#$%^&*()-+`\"\'", b"specials");
    create_entry(&root, "f2/with\nnewline", b"newline");

    create_entry(&root, "f3/inner1", b"");
    create_entry(&root, "f3/f4/inner2", b"");

    create_entry(&root, "f2/.hidden", b"hidden");
    create_entry(&root, "f2/with\ttab", b"tab");
    create_entry(&root, "f2/unicode_ハンバーガー_🍣", b"unicode");
    create_entry(&root, "empty_dir/", b"");

    create_entry(&root, "f4/inner2", b"another inner2");

    // Symlinks (created as working relative symlinks)
    // f5/sl1 -> f1/b.txt
    create_symlink(&root, "f5/sl1", "../f1/b.txt");
    // f5/sl2 -> f5/sl1
    create_symlink(&root, "f5/sl2", "sl1");
    // f5/f6/sl3 -> f5/
    create_symlink(&root, "f5/f6/sl3", "../..");

    // Optional: if you still want this here (personally I'd leave it to the test)
    let _ = write_tracking_file(&root);

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
///   find . -mindepth 1 -printf '%p\0' | xargs -0 -n1 printf '%q\n' | sort
///
/// Notes:
/// - Uses `sh -lc` because of the pipe.
fn find_escaped_output(dir: &std::path::Path) -> String {

    let cmd = r"find . -mindepth 1 -printf '%p\0' | xargs -0 -n1 printf '%q\n' | sort";
    let out = Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run shell command in '{}': {}", dir.display(), e));

    if !out.status.success() {
        panic!(
            "command failed in '{}': exit={:?}, stderr={}",
            dir.display(),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    String::from_utf8(out.stdout).unwrap_or_else(|e| panic!("baseline stdout not UTF-8: {}", e))
}
