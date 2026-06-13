use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

fn cargo_bin() -> Command {
    Command::cargo_bin("soupify").expect("binary should build")
}

#[test]
fn desoupify_accepts_a_direct_soup_file_path() {
    let temp = tempdir().expect("tempdir should exist");
    let restored = temp.path().join("nested/file.txt");
    let soup_file = temp.path().join("archive.soup");
    fs::write(
        &soup_file,
        format!(
            "#SOUP \"{}\" #SOUPED_LINES 2 #SOUP_TRAILING_NEWLINE 1\nhello\nworld",
            restored.display()
        ),
    )
    .expect("soup file should be written");

    cargo_bin().args(["-d"]).arg(&soup_file).assert().success();

    assert_eq!(
        fs::read_to_string(&restored).expect("restored file should exist"),
        "hello\nworld\n"
    );
}

#[test]
fn direct_soup_path_reports_parse_errors_from_that_file() {
    let temp = tempdir().expect("tempdir should exist");
    let soup_file = temp.path().join("broken.soup");
    fs::write(&soup_file, "#SOUP \"/tmp/file.txt\"").expect("soup file should be written");

    cargo_bin()
        .args(["-d"])
        .arg(&soup_file)
        .assert()
        .failure()
        .stderr(contains("malformed soup header"));
}
