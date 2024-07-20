use approx::assert_relative_eq;
use assert_cmd::prelude::*;
use itertools::Itertools;
use predicates::prelude::*;
use std::env;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn command_invalid() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("foobar");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("recognized"));

    Ok(())
}

#[test]
fn command_env() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("gams")?;
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
fn command_env_all() -> anyhow::Result<()> {
    let curdir = env::current_dir().unwrap();

    let tempdir = TempDir::new().unwrap();
    assert!(env::set_current_dir(&tempdir).is_ok());

    let mut cmd = Command::cargo_bin("gams")?;
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
fn command_env_env() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("gams")?;
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
fn command_status() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // test
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd.arg("status").arg("test").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.lines().count() > 20);
    assert!(stdout.contains("Running SET commands"));

    // dump
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd.arg("status").arg("dump").output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.lines().count() > 1);
    assert!(stderr.contains("Redis BGSAVE completed"));

    Ok(())
}

#[test]
fn command_libs_redis() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // get_vec_chr
    let mut conn = gams::Conn::new();
    let exp = vec!["I", "Mito"];
    let res = conn.get_vec_chr().into_iter().sorted().collect::<Vec<_>>();
    assert_eq!(res, exp, "get_vec_chr");

    // let exp = vec![
    //     "ctg:I:1",
    //     "ctg:I:2",
    //     "ctg:Mito:1",
    // ];
    // let res = gams::get_scan_vec(&mut conn, "ctg:*").into_iter().sorted().collect::<Vec<_>>();
    // assert_eq!(res.len(), exp.len());
    // assert_eq!(res, exp);

    Ok(())
}

#[test]
fn command_gen() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd
        .arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 17);
    assert!(stderr.contains("There are 3 contigs"));

    // get_scan_str
    let mut conn = gams::Conn::new();
    let exp = vec!["ctg:I:1", "ctg:I:2", "ctg:Mito:1"];
    let res = conn
        .get_scan_keys("ctg:*")
        .into_iter()
        .sorted()
        .collect::<Vec<_>>();
    assert_eq!(res.len(), exp.len());
    assert_eq!(res, exp);

    // // get_seq
    // let mut conn = gams::connect();
    // let tests = vec![
    //     ("I", 1000, 1002, "ATA"),
    //     ("I", 1000, 1010, "ATACAATTATA"),
    //     ("I", -1000, 1100, ""),
    //     ("II", 1000, 1100, ""),
    // ];
    // for (name, start, end, expected) in tests {
    //     let ctg = gams::get_rg_seq(&mut conn, &Range::from(name, start, end));
    //     assert_eq!(ctg, expected.to_string());
    // }
    //
    // // get_gc_content
    // let mut conn = gams::connect();
    // let tests = vec![
    //     ("I", 1000, 1002, 0.0),      // ATA
    //     ("I", 1000, 1010, 1. / 11.), // ATACAATTATA
    //     ("I", -1000, 1100, 0.0),
    //     ("II", 1000, 1100, 0.0),
    // ];
    // for (name, start, end, expected) in tests {
    //     let gc = gams::get_gc_content(&mut conn, &Range::from(name, start, end));
    //     assert_relative_eq!(gc, expected);
    // }

    Ok(())
}

#[test]
fn command_tsv() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // tsv
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd.arg("tsv").arg("-s").arg("ctg:*").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 4);
    assert_eq!(
        stdout.lines().next().unwrap().split('\t').count(),
        7,
        "field count"
    );
    assert!(stdout.contains("chr_strand\tlength"));
    assert!(stdout.contains("ctg:I:2"));

    Ok(())
}

#[test]
fn command_rg() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // range
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd
        .arg("rg")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("tests/S288c/spo11_hot.rg")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 3);
    assert!(stderr.contains("There are 69 rgs in this file"));

    Ok(())
}

#[test]
fn command_clear() -> anyhow::Result<()> {
    // gams env
    // gams status drop
    // gams gen tests/S288c/genome.fa.gz --piece 100000
    // gams range tests/S288c/spo11_hot.rg tests/S288c/spo11_hot.rg
    // gams clear range

    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // range
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("rg")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("tests/S288c/spo11_hot.rg")
        .output()
        .unwrap();

    // clear
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd.arg("clear").arg("rg").output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 4);
    assert!(stderr.contains("Clearing pattern \"rg:*\""));
    assert!(stderr.contains("Clear 2 keys"));

    Ok(())
}

#[test]
fn command_feature() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // feature
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd
        .arg("feature")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("--tag")
        .arg("spo11")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(stderr.lines().count(), 2);
    assert!(stderr.contains("There are 69 features in this file"));

    Ok(())
}

#[test]
fn command_fsw() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // feature
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("feature")
        .arg("tests/S288c/spo11_hot.rg")
        .arg("--tag")
        .arg("spo11")
        .output()
        .unwrap();

    // fsw
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd.arg("fsw").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.lines().count() > 2000);
    assert!(stdout.contains("fsw:feature:ctg:I:2:32:1"));

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 7);
    assert!(stderr.contains("Process ctg:I:2"));

    Ok(())
}

#[test]
fn command_sliding() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // sliding
    let mut cmd = Command::cargo_bin("gams")?;
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
fn command_sliding_p() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // sliding
    let mut cmd = Command::cargo_bin("gams")?;
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
        .arg("--parallel")
        .arg("2")
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
fn command_peak() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("500000")
        .unwrap();

    // peak
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd
        .arg("peak")
        .arg("tests/S288c/I.peaks.tsv")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("Process ctg:I:1"));

    Ok(())
}

#[test]
fn command_locate() -> anyhow::Result<()> {
    // gams env
    // gams status drop
    // gams gen tests/S288c/genome.fa.gz --piece 100000
    // gams locate "I:1000-1100" "II:1000-1100" "Mito:1000-1100"
    // gams locate -f tests/S288c/spo11_hot.rg

    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("500000")
        .unwrap();

    // locate
    let mut cmd = Command::cargo_bin("gams")?;
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

    // locate -f
    let mut cmd = Command::cargo_bin("gams")?;
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
fn command_anno() -> anyhow::Result<()> {
    // env
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("env").unwrap();

    // drop
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("status").arg("drop").unwrap();

    // gen
    let mut cmd = Command::cargo_bin("gams")?;
    cmd.arg("gen")
        .arg("tests/S288c/genome.fa.gz")
        .arg("--piece")
        .arg("100000")
        .unwrap();

    // anno
    let mut cmd = Command::cargo_bin("gams")?;
    let output = cmd
        .arg("anno")
        .arg("tests/S288c/intergenic.json")
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
        let (mean, stddev, cv) = gams::gc_stat(&gcs);
        assert_relative_eq!(mean, exp.0);
        assert_relative_eq!(stddev, exp.1);
        assert_relative_eq!(cv, exp.2);
    }
}
