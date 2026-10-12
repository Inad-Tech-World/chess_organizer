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
fn pairs_the_next_round_like_bbp_pairings() {
    // Same output as `bbpPairings --dutch sample.trf -p` (player 3 has a half-point bye).
    let out = co()
        .args(["pair", "--dutch"])
        .arg(sample())
        .arg("-p")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "3\n5 1\n2 4\n6 7\n");
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
