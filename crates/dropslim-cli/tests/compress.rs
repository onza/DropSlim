use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use image::{ImageBuffer, Rgba};
use predicates::prelude::*;
use tempfile::TempDir;

fn write_png(path: &std::path::Path, size: u32) {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(size, size, |x, y| Rgba([(x % 256) as u8, (y % 256) as u8, 80, 255]));
    img.save(path).expect("write png");
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn dropslim() -> Command {
    let mut cmd = Command::cargo_bin("dropslim").expect("dropslim binary");
    cmd.current_dir(repo_root());
    cmd
}

#[test]
fn compress_png_writes_min_suffix() {
    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("photo.png");
    write_png(&input, 64);

    dropslim()
        .args(["compress", input.to_str().unwrap()])
        .assert()
        .success();

    let output = dir.path().join("photo.min.png");
    assert!(output.is_file(), "expected {}", output.display());
    assert!(
        fs::metadata(&output).unwrap().len() > 0,
        "output should not be empty"
    );
}

#[test]
fn compress_json_emits_batch_complete() {
    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("photo.png");
    write_png(&input, 48);

    let assert = dropslim()
        .args(["compress", "--json", input.to_str().unwrap()])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("\"type\":\"batchComplete\""),
        "stdout was: {stdout}"
    );
}

#[test]
fn compress_missing_file_exits_one() {
    let dir = TempDir::new().expect("tempdir");
    let missing = dir.path().join("missing.png");

    dropslim()
        .args(["compress", "--quiet", missing.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("fileNotFound"));
}

#[test]
fn compress_out_dir_and_format() {
    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("photo.png");
    let out = dir.path().join("dest");
    fs::create_dir(&out).expect("mkdir out");
    write_png(&input, 32);

    dropslim()
        .args([
            "compress",
            "--quiet",
            "--format",
            "jpeg",
            "--out",
            out.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .assert()
        .success();

    let output = out.join("photo.min.jpg");
    assert!(output.is_file(), "expected {}", output.display());
}

#[test]
fn compress_requires_paths() {
    Command::cargo_bin("dropslim")
        .expect("dropslim binary")
        .args(["compress"])
        .assert()
        .failure()
        .code(2);
}
