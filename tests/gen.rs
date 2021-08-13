use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn command_status_drop() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("status")
        .arg("drop")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("OK"));

    Ok(())
}

#[test]
fn command_gen() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 14);
    assert!(stdout.contains("There are 3 contigs"));

    Ok(())
}

#[test]
fn command_status_dump() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("status")
        .arg("dump")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("OK"));

    Ok(())
}
