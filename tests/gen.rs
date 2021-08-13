use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs

#[test]
fn command_gen() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;

    // gen
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

    // find_one
    let tests = vec![
        ("I", 1000, 1100, "ctg:I:1"),
        ("Mito", 1000, 1100, "ctg:Mito:1"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = garr::find_one(name, start, end);
        assert_eq!(ctg, expected.to_string());
    }

    Ok(())
}
