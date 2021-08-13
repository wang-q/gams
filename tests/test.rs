use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn command_status_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("status")
        .arg("test")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 20);
    assert!(stdout.contains("Running SET commands"));

    Ok(())
}
