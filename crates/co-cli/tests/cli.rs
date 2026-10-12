use std::path::PathBuf;
use std::process::Command;

fn co() -> Command {
    Command::new(env!("CARGO_BIN_EXE_co"))
}

fn sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/trf/sample.trf")
}

#[test]
fn trf_survives_a_trip_through_a_cot_file() {
    let dir = tempfile::tempdir().unwrap();
    let cot = dir.path().join("t.cot");
    let back = dir.path().join("back.trf");
    assert!(
        co().args(["convert"])
            .arg(sample())
            .arg(&cot)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        co().args(["convert"])
            .arg(&cot)
            .arg(&back)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(sample()).unwrap(),
        std::fs::read_to_string(back).unwrap()
    );
}

#[test]
fn renders_standings() {
    let out = co()
        .args(["report", "standings"])
        .arg(sample())
        .args(["--format", "csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let csv = String::from_utf8(out.stdout).unwrap();
    assert!(csv.lines().nth(1).unwrap().contains("Okello, David"));
}

#[test]
fn planned_features_exit_with_code_3() {
    let out = co()
        .args(["pair", "--dutch"])
        .arg(sample())
        .arg("-p")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Phase 1"));
}

#[test]
fn check_accepts_sample() {
    let out = co()
        .args(["check", "--dutch"])
        .arg(sample())
        .arg("-c")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
