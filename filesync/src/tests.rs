use crate::{read_tracking_file_into_filepaths, read_tracking_file_into_string, run, write_tracking_file, write_tracking_file_with_content, ProgramArgs, TRACKING_FILENAME};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::os::unix::fs as unix_fs;  // we're supporting unix filesystem features such as symlinks
use std::process::Command;
use clap::Parser;
use rayon::prelude::*;


use test_case::test_case;
use crate::structures::ManifestEntry;


#[test_case("$HOME/Downloads")]
fn test_write_tracking_file(dir_spec: &str) {
    let base_dir = expand_home(dir_spec);

    // write empty file
    let (_, mut file) = write_tracking_file(&base_dir);  // if this doesn't panic, we're good

    fn assert_file_empty(file: &fs::File) { assert_eq!(file.metadata().expect("can't check file metadata!").len(), 0) }
    fn assert_file_non_empty(file: &fs::File) { assert!(file.metadata().expect("can't check file metadata!!").len() > 0) }


    file.set_len(0).expect(""); // clears content
    assert_file_empty(&file);

    let our_string = "AAAAAAAAAAAAAAAAAAAAAAABBBBBBBBBBBBBBB\n";

    // make sure that writing the file doesn't overwrite it, if it exists
    file.write_all(our_string.as_bytes()).expect("failed to write to file");
    file.flush().expect("failed to flush"); // harmless for File; required if buffered somewhere
    assert_file_non_empty(&file);

    let (same_path, same_file) = write_tracking_file(&base_dir);  // writing again
    assert_file_non_empty(&same_file);  // checking file wasn't overwritten
    assert!(read_tracking_file_into_string(&same_path).contains(our_string));

    let filled_file_path = write_tracking_file_with_content(&base_dir, None);  // rewrite file contents
    assert!(!read_tracking_file_into_string(&filled_file_path).contains(our_string));  // make sure previous string is overwritten

    let _ = fs::remove_file(&filled_file_path);  // cleanup - remove tracking-file
}


#[test]
fn tracking_file_compare_with_shell_command() {
    let tracker_content = create_tree_and_tracker_and_read_paths("S", None);
    let baseline_out = find_escaped_output(&define_tmp_dir("S"));

    assert_eq!(tracker_content, baseline_out);
}

#[test]
fn check_serialized_deserialization_is_same() {
    let tracker_filepath = create_tree_and_tracker("serialization_test", None);

    let file_content: String = read_tracking_file_into_string(&tracker_filepath);

    // deserialize from String, then serialize into String
    let undeserailized = ManifestEntry::serialize_manifests(ManifestEntry::deserialize_manifests(&file_content).as_slice());

    assert_eq!(file_content.lines().collect::<Vec<_>>(), undeserailized);
}


#[test]
fn test_args_cli_track() {

    fn run_w_args(args: &[&str]) -> String { run(ProgramArgs::parse_from(args)) }

    let root = creates_complicated_testing_tree("CLI", None);
    // let tracker = run_w_args(&["filesync", "--track", expand_home("$HOME/Downloads").to_str().unwrap()]);
    // let tracker1 = run_w_args(&["filesync", "--track", &root.to_str().unwrap()]);
    let tracker2 = run_w_args(&["filesync", "--track", &root.to_str().unwrap(), "-p", "f3"]);


}

/// tracking with specific prefixes (rather than all files)
#[test]
fn test_picked_track_scans() {
    // TODO - pick a subdir, check that only those got "walked"
    // TODO - pick another subdir, check that those got walked and added (not replacing previous)
    // TODO - move a file, rescan, and see that the tracking file got updated (erasing relevant previous entries)
    // TODO - check unicode prefixes
}


// #[test]
fn detect_differences_between_filetrees() {
    // need to read only paths because the creation date would be different
    let tracker_content_a = create_tree_and_tracker_and_read_paths("A", None);
    let extras: Vec<String> = vec!["EXTRA/x.txt".into(), "EXTRA/y.txt".into()];
    let tracker_content_b = create_tree_and_tracker_and_read_paths("B", Some(&extras));  // B has "extra" files

    assert_eq!(tracker_content_a, tracker_content_b);
}


/// returns the path of the newly created tracking file
fn create_tree_and_tracker(subdir: &str, extra: Option<&[String]>) -> PathBuf {
    let new_dir = creates_complicated_testing_tree(subdir, extra);
    write_tracking_file_with_content(&new_dir, None)
}

/// returns the newly made and listed files within the new tracking file
fn create_tree_and_tracker_and_read_paths(subdir: &str, extra: Option<&[String]>) -> Vec<String> {
    let tracker_filepath = create_tree_and_tracker(subdir, extra);
    read_tracking_file_into_filepaths(&tracker_filepath)
}


fn define_tmp_dir(subdir: &str) -> PathBuf {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    project_root.join("testing").join(subdir)
}

fn creates_complicated_testing_tree(subdir: &str, extra: Option<&[String]>) -> PathBuf {
    let root = define_tmp_dir(subdir);

    let _ = fs::remove_dir_all(&root);

    // simple
    create_entry(&root, "f1/a.txt", b"");
    create_entry(&root, "f1/b.txt", b"hello world");

    // problematic chars
    create_entry(&root, "f2/a.txt", b"another a");
    create_entry(&root, "f2/ with    space", b"space");
    create_entry(&root, "f2/special!@#$%^&*()-+`\"\'", b"specials");
    create_entry(&root, "f2/with\nnewline", b"newline");
    create_entry(&root, "f2/with\ttab", b"tab");
    create_entry(&root, "f2/unicode_ハンバーガー_🍣", b"unicode");
    create_entry(&root, "f2/unicode_ハハハハハハハハハ", b"unicode");
    create_entry(&root, "f2/unicode_🍣🍣🍣🍣🍣🍣🍣🍣", b"unicode");
    create_entry(&root, "f2/emojis_🇺🇸🇺🇸🇺🇸🇺🇸🇺🇸🇺🇸🇺🇸", b"unicode");
    create_entry(&root, "f2/escaped_\\\'\"\'\'\\\\\t\\\'", b"unicode");
    create_entry(&root, "ハwハwハ", b"unicode");

    // hierarchy/hidden
    create_entry(&root, "f3/inner1", b"");
    create_entry(&root, "f3/f4/inner2", b"inner2");
    create_entry(&root, "f3/.hidden", b"hidden");

    // empty
    create_entry(&root, "empty_dir/", b"");  // empty dir
    create_entry(&root, "empty_file", b"");  // empty file in root

    // extra instances
    create_entry(&root, "f4/inner2", b"another inner2");  // "duplicate" file
    create_entry(&root, &format!("f4/{}", TRACKING_FILENAME), b"another inner2");  // "duplicate" file

    // links
    create_symlink(&root, "f5/sl1", "../f1/b.txt");
    create_symlink(&root, "f5/sl2", "sl1");  // f5/sl2 -> f5/sl1
    create_symlink(&root, "f5/f6/sl3", "../..");  // f5/f6/sl3 -> f5/
    create_symlink(&root, "f5/f6/sl4", expand_home("$HOME/Downloads").to_str().unwrap());  // link outside of project

    // extras (optional)
    if let Some(extra) = extra {
        for rel in extra {
            create_entry(&root, rel, b"");
        }
    }

    root
}


fn expand_home(s: &str) -> PathBuf {
    if s.starts_with("$HOME/") || s == "$HOME" {
        let home = env::var("HOME").expect("HOME is not set");
        let rest = s.strip_prefix("$HOME").unwrap();
        return PathBuf::from(home).join(rest.trim_start_matches('/'));
    }
    PathBuf::from(s)
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
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
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
    let mut lines: Vec<String> = Command::new("sh")  // run "find" command
        .arg("-lc")
        .arg(r"find . -mindepth 1 -printf '%P\0'")
        .current_dir(dir)
        .output()       // shell command output
        .inspect_err(|e| panic!("failed to run shell command in '{}': {}", dir.display(), e))
        .inspect( |out| assert!(out.status.success(), "find failed in '{}': exit={:?}, stderr={}", dir.display(), out.status.code(), String::from_utf8_lossy(&out.stderr),) )
        .unwrap().stdout.par_split(|&b| b == 0)  // split stdout on \0
        .filter(|s| !s.is_empty())  // sanitize
        .filter(|s| *s != TRACKING_FILENAME.as_bytes())  // sanitize
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();

    lines.par_sort_unstable();  // no need to preserve equals' order; run it a bit faster
    lines
}

