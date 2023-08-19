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
    let mut conn = gars::connect();

    for _ in 0..size {
        let _: () = conn
            .set(format!("prefix:{}", rand_str(4)), rand_str(16))
            .unwrap();
        let _: () = conn.set(format!("{}", rand_str(8)), rand_str(16)).unwrap();
    }
}

pub fn bench_redis_scan(c: &mut Criterion) {
    let mut conn = gars::connect();

    gars::db_drop();
    rand_insert(black_box(5000));

    c.bench_function("scan_match_10", |b| {
        b.iter(|| {
            let _: Vec<_> = gars::get_scan_match_vec(&mut conn, "prefix:*");
        })
    });
    c.bench_function("scan_count_10", |b| {
        b.iter(|| {
            let _: Vec<_> = gars::get_scan_vec(&mut conn, "prefix:*", 10);
        })
    });
    c.bench_function("scan_count_100", |b| {
        b.iter(|| {
            let _: Vec<_> = gars::get_scan_vec(&mut conn, "prefix:*", 100);
        })
    });
    c.bench_function("scan_count_1000", |b| {
        b.iter(|| {
            let _: Vec<_> = gars::get_scan_vec(&mut conn, "prefix:*", 1000);
        })
    });
    c.bench_function("scan_count_10000", |b| {
        b.iter(|| {
            let _: Vec<_> = gars::get_scan_vec(&mut conn, "prefix:*", 10000);
        })
    });
}

criterion_group!(benches, bench_redis_scan);
criterion_main!(benches);
