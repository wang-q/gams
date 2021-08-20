use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs // f32

#[test]
fn command_wave() -> Result<(), Box<dyn std::error::Error>> {
    // drop
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // range
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("wave")
        .arg("tests/S288c/I.peaks.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("There are 117 peaks"));

    Ok(())
}
