use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use image::{ImageBuffer, Rgba};
use predicates::prelude::*;
use tempfile::TempDir;

fn write_png(path: &Path, size: u32) {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(size, size, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, 80, 255])
    });
    img.save(path).expect("write png");
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures_dir() -> PathBuf {
    repo_root().join("test").join("fixtures")
}

fn gifsicle_bin() -> Option<PathBuf> {
    let binary = if cfg!(windows) {
        "gifsicle.exe"
    } else {
        "gifsicle"
    };
    let path = repo_root().join("vendor").join("gifsicle").join(binary);
    if path.is_file() {
        Some(path)
    } else {
        eprintln!(
            "skip: gifsicle not found at {} — run npm ci to install vendor binaries",
            path.display()
        );
        None
    }
}

fn dropslim() -> Command {
    let mut cmd = Command::cargo_bin("dropslim").expect("dropslim binary");
    cmd.current_dir(repo_root());
    cmd
}

fn copy_fixture(name: &str, dest: &Path) {
    fs::copy(fixtures_dir().join(name), dest)
        .unwrap_or_else(|error| panic!("copy fixture {name} → {}: {error}", dest.display()));
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

#[test]
fn compress_subfolder_writes_minified() {
    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("photo.png");
    write_png(&input, 40);

    dropslim()
        .args([
            "compress",
            "--quiet",
            "--subfolder",
            input.to_str().unwrap(),
        ])
        .assert()
        .success();

    let output = dir.path().join("minified").join("photo.min.png");
    assert!(output.is_file(), "expected {}", output.display());
    assert!(!dir.path().join("photo.min.png").exists());
}

#[test]
fn compress_no_suffix_overwrites_in_place() {
    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("photo.png");
    write_png(&input, 56);
    let before = fs::metadata(&input).expect("input metadata").len();

    dropslim()
        .args([
            "compress",
            "--quiet",
            "--no-suffix",
            input.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(input.is_file(), "input should still exist after overwrite");
    assert!(!dir.path().join("photo.min.png").exists());
    let after = fs::metadata(&input).expect("output metadata").len();
    assert!(after > 0);
    assert!(
        after <= before,
        "overwrite should not grow file ({after} > {before})"
    );
}

#[test]
fn compress_folder_batch_optimizes_all_pngs() {
    let dir = TempDir::new().expect("tempdir");
    let nested = dir.path().join("shots");
    fs::create_dir(&nested).expect("mkdir");
    write_png(&nested.join("a.png"), 24);
    write_png(&nested.join("b.png"), 28);
    fs::write(dir.path().join("readme.txt"), b"not an image").expect("write txt");

    let assert = dropslim()
        .args([
            "compress",
            "--quiet",
            "--json",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(nested.join("a.min.png").is_file());
    assert!(nested.join("b.min.png").is_file());

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("\"succeeded\":2"),
        "expected 2 successes in {stdout}"
    );
}

#[test]
fn compress_gif_fixture_when_gifsicle_available() {
    let Some(_) = gifsicle_bin() else {
        return;
    };

    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("bloat.gif");
    copy_fixture("bloat.gif", &input);
    let before = fs::metadata(&input).expect("gif metadata").len();

    dropslim()
        .args(["compress", "--quiet", input.to_str().unwrap()])
        .assert()
        .success();

    let output = dir.path().join("bloat.min.gif");
    assert!(output.is_file(), "expected {}", output.display());
    let after = fs::metadata(&output).expect("output metadata").len();
    assert!(after > 0);
    assert!(
        after < before,
        "expected gif to shrink ({after} >= {before})"
    );
}

#[test]
fn compress_heic_fixture_on_macos() {
    if !cfg!(target_os = "macos") {
        eprintln!("skip: HEIC CLI path is macOS-only");
        return;
    }

    let heic = fixtures_dir().join("sample.heic");
    if !heic.is_file() {
        eprintln!("skip: missing test/fixtures/sample.heic");
        return;
    }

    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("sample.heic");
    copy_fixture("sample.heic", &input);

    dropslim()
        .args(["compress", "--quiet", input.to_str().unwrap()])
        .assert()
        .success();

    let output = dir.path().join("sample.min.heic");
    assert!(output.is_file(), "expected {}", output.display());
    assert!(fs::metadata(&output).unwrap().len() > 0);
}

#[test]
fn compress_heic_to_jpeg_on_macos() {
    if !cfg!(target_os = "macos") {
        eprintln!("skip: HEIC convert is macOS-only");
        return;
    }

    let heic = fixtures_dir().join("sample.heic");
    if !heic.is_file() {
        eprintln!("skip: missing test/fixtures/sample.heic");
        return;
    }

    let dir = TempDir::new().expect("tempdir");
    let input = dir.path().join("sample.heic");
    let out = dir.path().join("dest");
    fs::create_dir(&out).expect("mkdir");
    copy_fixture("sample.heic", &input);

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

    let output = out.join("sample.min.jpg");
    assert!(output.is_file(), "expected {}", output.display());
}
