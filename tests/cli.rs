use approx::assert_relative_eq;
use assert_cmd::prelude::*; // Add methods on commands
use intspan::*;
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn command_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("foobar");
    cmd.assert().failure().stderr(predicate::str::contains(
        "which wasn't expected, or isn't valid in this context",
    ));

    Ok(())
}

#[test]
fn command_env() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("env")
        .arg("--outfile")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 6);
    assert!(stdout.contains("REDIS_PASSWORD=''"), "original values");

    Ok(())
}

#[test]
fn command_env_env() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .env("REDIS_PASSWORD", "mYpa$$")
        .arg("env")
        .arg("--outfile")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 6);
    assert!(
        stdout.contains("REDIS_PASSWORD='mYpa$$'"),
        "modified values"
    );

    Ok(())
}

#[test]
fn command_status() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("status").arg("drop").unwrap();

    // test
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd.arg("status").arg("test").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 20);
    assert!(stdout.contains("Running SET commands"));

    // dump
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd.arg("status").arg("dump").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("OK"));

    Ok(())
}

#[test]
fn command_gen() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
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

    // find_one
    let mut conn = garr::connect();
    let tests = vec![
        ("I", 1000, 1100, "ctg:I:1"),
        ("Mito", 1000, 1100, "ctg:Mito:1"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = garr::find_one(&mut conn, &Range::from(name, start, end));
        assert_eq!(ctg, expected.to_string());
    }

    // get_seq
    let mut conn = garr::connect();
    let tests = vec![
        ("I", 1000, 1002, "ATA"),
        ("I", 1000, 1010, "ATACAATTATA"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = garr::get_seq(&mut conn, &Range::from(name, start, end));
        assert_eq!(ctg, expected.to_string());
    }

    // get_gc_content
    let mut conn = garr::connect();
    let tests = vec![
        ("I", 1000, 1002, 0.0),      // ATA
        ("I", 1000, 1010, 1. / 11.), // ATACAATTATA
        ("I", -1000, 1100, 0.0),
        ("II", 1000, 1100, 0.0),
    ];
    for (name, start, end, expected) in tests {
        let gc = garr::get_gc_content(&mut conn, &Range::from(name, start, end));
        assert_relative_eq!(gc, expected);
    }

    Ok(())
}

#[test]
fn command_pos() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("env").unwrap();

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

    // pos
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("pos")
        .arg("tests/S288c/spo11_hot.pos.txt")
        .arg("tests/S288c/spo11_hot.pos.txt")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("71 positions in total"));
    assert!(stdout.contains("142 positions in total"));

    Ok(())
}

#[test]
fn command_range() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("env").unwrap();

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
        .arg("range")
        .arg("tests/S288c/spo11_hot.pos.txt")
        .arg("--tag")
        .arg("spo11")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("There are 71 ranges"));

    Ok(())
}

#[test]
fn command_wave() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("garr")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("500000")
        .unwrap();

    // wave
    let mut cmd = Command::cargo_bin("garr")?;
    let output = cmd
        .arg("wave")
        .arg("tests/S288c/I.peaks.tsv")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("There are 155 peaks"));

    Ok(())
}

#[test]
fn test_gc_stat() {
    let tests = vec![
        (vec![0.5, 0.5], (0.5, 0., 0., 0.)),
        (
            vec![0.4, 0.5, 0.5, 0.6],
            (0.5, 0.08164966, 0.16329932, 6.123724),
        ),
    ];
    for (gcs, exp) in tests {
        let (mean, stddev, cv, snr) = garr::gc_stat(&gcs);
        assert_relative_eq!(mean, exp.0);
        assert_relative_eq!(stddev, exp.1);
        assert_relative_eq!(cv, exp.2);
        assert_relative_eq!(snr, exp.3);
    }
}
