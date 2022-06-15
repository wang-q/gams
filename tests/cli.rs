use approx::assert_relative_eq;
use assert_cmd::prelude::*; // Add methods on commands
use intspan::*;
use predicates::prelude::*; // Used for writing assertions
use std::collections::HashMap;
use std::env;
use std::process::Command; // Run programs
use tempfile::TempDir;

#[test]
fn command_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("foobar");
    cmd.assert().failure().stderr(predicate::str::contains(
        "which wasn't expected, or isn't valid in this context",
    ));

    Ok(())
}

#[test]
fn command_env() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("env")
        .arg("--outfile")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("6379"), "original values");

    Ok(())
}

#[test]
fn command_env_all() -> Result<(), Box<dyn std::error::Error>> {
    let curdir = env::current_dir().unwrap();

    let tempdir = TempDir::new().unwrap();
    assert!(env::set_current_dir(&tempdir).is_ok());

    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("env")
        .arg("--all")
        .arg("--outfile")
        .arg("stdout")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 9);
    assert!(stderr.contains("Create plot_xy.R"));
    assert!(stderr.contains("Create sqls/summary.sql"));

    // cleanup
    assert!(env::set_current_dir(&curdir).is_ok());
    assert!(tempdir.close().is_ok());

    Ok(())
}

#[test]
fn command_env_env() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .env("REDIS_PORT", "7379")
        .arg("env")
        .arg("--outfile")
        .arg("stdout")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("REDIS_PORT=7379"), "modified values");

    Ok(())
}

#[test]
fn command_status() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // test
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd.arg("status").arg("test").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 20);
    assert!(stdout.contains("Running SET commands"));

    // dump
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd.arg("status").arg("dump").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("OK"));

    Ok(())
}

#[test]
fn command_gen() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 16);
    assert!(stderr.contains("There are 3 contigs"));

    // get_scan_str
    let mut conn = gars::connect();
    let exp: HashMap<String, String> = HashMap::from([
        ("ctg:I:1".to_string(), "I".to_string()),
        ("ctg:I:2".to_string(), "I".to_string()),
        ("ctg:Mito:1".to_string(), "Mito".to_string()),
    ]);
    let res = gars::get_scan_str(&mut conn, "ctg:*".to_string(), "chr_id".to_string());
    assert_eq!(res.len(), exp.len());
    assert!(res.keys().all(|k| exp.contains_key(k)));
    assert!(res
        .keys()
        .all(|k| res.get(k).unwrap() == exp.get(k).unwrap()));

    // get_scan_int
    let mut conn = gars::connect();
    let exp: HashMap<String, i32> = HashMap::from([
        ("ctg:I:1".to_string(), 100000),
        ("ctg:I:2".to_string(), 230218),
        ("ctg:Mito:1".to_string(), 85779),
    ]);
    let res = gars::get_scan_int(&mut conn, "ctg:*".to_string(), "chr_end".to_string());
    assert_eq!(res.len(), exp.len());
    assert!(res.keys().all(|k| exp.contains_key(k)));
    assert!(res
        .keys()
        .all(|k| res.get(k).unwrap() == exp.get(k).unwrap()));

    // find_one_z
    let mut conn = gars::connect();
    let tests = vec![
        ("I", 1000, 1100, "ctg:I:1"),
        ("Mito", 1000, 1100, "ctg:Mito:1"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = gars::find_one_z(&mut conn, &Range::from(name, start, end));
        assert_eq!(ctg, expected.to_string());
    }

    // find_one_l
    let mut conn = gars::connect();
    let tests = vec![
        ("I", 1000, 1100, "ctg:I:1"),
        ("Mito", 1000, 1100, "ctg:Mito:1"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = gars::find_one_l(&mut conn, &Range::from(name, start, end));
        assert_eq!(ctg, expected.to_string());
    }

    // get_seq
    let mut conn = gars::connect();
    let tests = vec![
        ("I", 1000, 1002, "ATA"),
        ("I", 1000, 1010, "ATACAATTATA"),
        ("I", -1000, 1100, ""),
        ("II", 1000, 1100, ""),
    ];
    for (name, start, end, expected) in tests {
        let ctg = gars::get_rg_seq(&mut conn, &Range::from(name, start, end));
        assert_eq!(ctg, expected.to_string());
    }

    // get_gc_content
    let mut conn = gars::connect();
    let tests = vec![
        ("I", 1000, 1002, 0.0),      // ATA
        ("I", 1000, 1010, 1. / 11.), // ATACAATTATA
        ("I", -1000, 1100, 0.0),
        ("II", 1000, 1100, 0.0),
    ];
    for (name, start, end, expected) in tests {
        let gc = gars::get_gc_content(&mut conn, &Range::from(name, start, end));
        assert_relative_eq!(gc, expected);
    }

    Ok(())
}

#[test]
fn command_tsv() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // tsv
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd.arg("tsv").arg("-s").arg("ctg:*").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        6,
        "field count"
    );
    assert!(stdout.contains("chr_strand\tlength"));
    assert!(stdout.contains("ctg:I:2"));

    // tsv -f length
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("tsv")
        .arg("-s")
        .arg("ctg:*")
        .arg("-f")
        .arg("length")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        5,
        "field count"
    );
    assert!(!stdout.contains("chr_strand\tlength"));
    assert!(stdout.contains("chr_end\tlength"));
    assert!(stdout.contains("ctg:I:2"));

    Ok(())
}

#[test]
fn command_range() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // range
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("range")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("tests/S288c/spo11_hot.rg")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 2);
    assert!(stderr.contains("71 ranges in total"));
    assert!(stderr.contains("142 ranges in total"));

    Ok(())
}

#[test]
fn command_clear() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // range
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("range")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("tests/S288c/spo11_hot.rg")
        .output()
        .unwrap();

    // clear
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd.arg("clear").arg("range").output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 4);
    assert!(stderr.contains("Clearing pattern \"range:*\""));
    assert!(stderr.contains("Clear 2 keys"));

    Ok(())
}

#[test]
fn command_feature() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // feature
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("feature")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("--tag")
        .arg("spo11")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.contains("There are 71 features"));

    Ok(())
}

#[test]
fn command_fsw() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // feature
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("feature")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("--tag")
        .arg("spo11")
        .output()
        .unwrap();

    // fsw
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd.arg("fsw").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2867);
    assert!(stdout.contains("fsw:feature:ctg:I:2:32:1"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 7);
    assert!(stderr.contains("Process ctg:I:2"));

    Ok(())
}

#[test]
fn command_sliding() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // sliding
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("sliding")
        .arg("--ctg")
        .arg("ctg:I:*")
        .arg("--size")
        .arg("100")
        .arg("--step")
        .arg("10")
        .arg("--lag")
        .arg("100")
        .arg("--threshold")
        .arg("3.0")
        .arg("--influence")
        .arg("1.0")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 23004);
    assert!(stdout.contains("I:78291-78390\t0.39"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 3);
    assert!(stderr.contains("Process ctg:I:2"));

    Ok(())
}

#[test]
fn command_peak() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("500000")
        .unwrap();

    // peak
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("peak")
        .arg("tests/S288c/I.peaks.tsv")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("There are 155 peaks"));

    Ok(())
}

#[test]
fn command_locate() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("500000")
        .unwrap();

    // locate
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("locate")
        .arg("I:1000-1100")
        .arg("II:1000-1100")
        .arg("Mito:1000-1100")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("ctg:I:1"));
    assert!(!stdout.contains("II:1000-1100"));

    // locate --lapper
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("locate")
        .arg("--lapper")
        .arg("I:1000-1100")
        .arg("II:1000-1100")
        .arg("Mito:1000-1100")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("ctg:I:1"));
    assert!(!stdout.contains("II:1000-1100"));

    // locate --zrange
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("locate")
        .arg("--zrange")
        .arg("I:1000-1100")
        .arg("II:1000-1100")
        .arg("Mito:1000-1100")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("ctg:I:1"));
    assert!(!stdout.contains("II:1000-1100"));

    // locate -f
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("locate")
        .arg("-f")
        .arg("tests/S288c/spo11_hot.rg")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 71);
    assert!(stdout.contains("ctg:I:1"));
    assert!(!stdout.contains("ctg:Mito:1"));

    Ok(())
}

#[test]
fn command_anno() -> Result<(), Box<dyn std::error::Error>> {
    // env
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gars")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // anno
    let mut cmd = Command::cargo_bin("gars")?;
    let output = cmd
        .arg("anno")
        .arg("tests/S288c/intergenic.yml")
        .arg("tests/S288c/ctg.range.tsv")
        .arg("-H")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        8,
        "field count"
    );
    assert!(stdout.contains("85779\t0.0000"));
    assert!(stdout.contains("130218\t0.1072"));

    Ok(())
}

#[test]
fn test_gc_stat() {
    let tests = vec![
        (vec![0.5, 0.5], (0.5, 0., 0.)),
        (vec![0.4, 0.5, 0.5, 0.6], (0.5, 0.08164966, 0.16329932)),
    ];
    for (gcs, exp) in tests {
        let (mean, stddev, cv) = gars::gc_stat(&gcs);
        assert_relative_eq!(mean, exp.0);
        assert_relative_eq!(stddev, exp.1);
        assert_relative_eq!(cv, exp.2);
    }
}
