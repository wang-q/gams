use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::distributions::{Alphanumeric, DistString};
use redis::Commands;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Ctg {
    pub id: String,
    pub range: String,
    pub chr_id: String,
    pub chr_start: i32,
    pub chr_end: i32,
    pub chr_strand: String,
    pub length: i32,
}

fn rand_str(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}

fn rand_insert(size: usize) {
    let mut conn = gams::connect();

    for _ in 0..size {
        let _: () = conn
            .set(format!("prefix:{}", rand_str(4)), rand_str(16))
            .unwrap();
        let _: () = conn.set(format!("{}", rand_str(8)), rand_str(16)).unwrap();
    }
}

/// Default COUNT is 10
/// Don't use this
fn get_scan_match_vec(conn: &mut redis::Connection, scan: &str) -> Vec<String> {
    // number of matches
    let mut keys: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match(scan).unwrap();
    for x in iter {
        keys.push(x);
    }

    keys
}

fn get_scan_vec_n(conn: &mut redis::Connection, pattern: &str, count: usize) -> Vec<String> {
    // matched keys
    let mut keys: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = redis::cmd("SCAN")
        .cursor_arg(0)
        .arg("MATCH")
        .arg(pattern)
        .arg("COUNT")
        .arg(count) // default is 10
        .clone()
        .iter(conn)
        .unwrap();
    for x in iter {
        keys.push(x);
    }

    keys
}

fn get_scan_vec(conn: &mut redis::Connection, pattern: &str) -> Vec<String> {
    get_scan_vec_n(conn, pattern, 1000)
}

pub fn bench_redis_scan(c: &mut Criterion) {
    let mut conn = gams::connect();

    gams::db_drop();
    rand_insert(black_box(5000));

    c.bench_function("scan_count", |b| {
        b.iter(|| {
            let n: i32 = gams::get_scan_count(&mut conn, "prefix:*");
            assert_eq!(n, 5000);
        })
    });
    c.bench_function("scan_lua", |b| {
        b.iter(|| {
            let vec: Vec<_> = gams::get_scan_keys(&mut conn, "prefix:*");
            assert_eq!(vec.len(), 5000);
        })
    });
    c.bench_function("scan_match_10", |b| {
        b.iter(|| {
            let vec: Vec<_> = get_scan_match_vec(&mut conn, "prefix:*");
            assert_eq!(vec.len(), 5000);
        })
    });
    c.bench_function("scan_count_10", |b| {
        b.iter(|| {
            let vec: Vec<_> = get_scan_vec_n(&mut conn, "prefix:*", 10);
            assert_eq!(vec.len(), 5000);
        })
    });
    c.bench_function("scan_count_100", |b| {
        b.iter(|| {
            let _: Vec<_> = get_scan_vec_n(&mut conn, "prefix:*", 100);
        })
    });
    c.bench_function("scan_count_1000", |b| {
        b.iter(|| {
            let _: Vec<_> = get_scan_vec_n(&mut conn, "prefix:*", 1000);
        })
    });
    c.bench_function("scan_count_10000", |b| {
        b.iter(|| {
            let _: Vec<_> = get_scan_vec_n(&mut conn, "prefix:*", 10000);
        })
    });
}

criterion_group!(benches, bench_redis_scan);
criterion_main!(benches);
