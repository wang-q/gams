use approx::assert_relative_eq;
use assert_cmd::prelude::*; // Add methods on commands
use std::process::Command; // Run programs // f32

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
    let mut conn = garr::connect();
    let tests = vec![
        ("I", 1000, 1100, "ctg:I:1"),
        ("Mito", 1000, 1100, "ctg:Mito:1"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = garr::find_one(&mut conn, name, start, end);
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
        let ctg = garr::get_seq(&mut conn, name, start, end);
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
        let gc = garr::get_gc_content(&mut conn, name, start, end);
        assert_relative_eq!(gc, expected);
    }

    Ok(())
}

#[test]
fn t_gc_stat() {
    let tests = vec![
        (vec![0.5, 0.5], (0.5, 0., 0., 0.)),
        (
            vec![0.4, 0.5, 0.5, 0.6],
            (0.5, 0.08164966, 0.16329932, 6.123724),
        ),
    ];
    for (gcs, exp) in tests {
        let (mean, stddev, cv, snr) = gc_stat(&gcs);
        assert_relative_eq!(mean, exp.0);
        assert_relative_eq!(stddev, exp.1);
        assert_relative_eq!(cv, exp.2);
        assert_relative_eq!(snr, exp.3);
    }
}
