use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::{path::Path, process::Command}; // Run programs

#[test]
fn convert_test_file() {
    let mut cmd = Command::cargo_bin("aag2courageous").unwrap();

    let test_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/1");
    let verification_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/1.json");
    let test_result_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("test.json");
    cmd.arg(&test_path)
        .arg("0,0,0")
        .arg("-o")
        .arg(&test_result_path);
    cmd.assert().success();

    assert!(predicate::path::eq_file(&verification_path).eval(test_result_path.as_path()));
}

#[test]
fn missing_cuas_location() {
    let mut cmd = Command::cargo_bin("aag2courageous").unwrap();

    let test_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/1");
    let test_result_path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("test.json");
    cmd.arg(&test_path).arg("-o").arg(&test_result_path);
    cmd.assert().failure();
}
