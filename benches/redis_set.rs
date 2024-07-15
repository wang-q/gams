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

fn rand_ctg() -> Ctg {
    Ctg {
        id: rand_str(8),
        range: rand_str(12),
        chr_id: rand_str(8),
        chr_start: rand::random::<i32>(),
        chr_end: rand::random::<i32>(),
        chr_strand: "+".to_string(),
        length: rand::random::<i32>(),
    }
}

pub fn bench_redis_set(c: &mut Criterion) {
    let mut conn = gams::connect();
    let ctg = rand_ctg();

    gams::db_drop();
    c.bench_function("redis_hset", |b| {
        b.iter(|| {
            // a short length makes the total number of randomized combinations not too large
            let ctg_id = rand_str(4);
            let _: () = conn
                .hset(black_box(&ctg_id), "range", black_box(&ctg.range))
                .unwrap();
            let _: () = conn
                .hset(black_box(&ctg_id), "chr_id", black_box(&ctg.chr_id))
                .unwrap();
            let _: () = conn
                .hset(black_box(&ctg_id), "chr_start", black_box(&ctg.chr_start))
                .unwrap();
            let _: () = conn
                .hset(black_box(&ctg_id), "chr_end", black_box(&ctg.chr_end))
                .unwrap();
            let _: () = conn
                .hset(black_box(&ctg_id), "chr_strand", black_box(&ctg.chr_strand))
                .unwrap();
            let _: () = conn
                .hset(black_box(&ctg_id), "length", black_box(&ctg.length))
                .unwrap();
        })
    });

    gams::db_drop();
    c.bench_function("redis_set_bincode", |b| {
        b.iter(|| {
            let ctg_id = rand_str(4);
            let bytes = bincode::serialize(black_box(&ctg)).unwrap();
            let _: () = conn.set(black_box(&ctg_id), &bytes).unwrap();
        })
    });

    gams::db_drop();
    c.bench_function("redis_hset_multiple", |b| {
        b.iter(|| {
            let ctg_id = rand_str(4);
            let _: () = conn
                .hset_multiple(
                    black_box(&ctg_id),
                    &[
                        ("range", black_box(&ctg.range)),
                        ("chr_id", black_box(&ctg.chr_id)),
                        ("chr_strand", black_box(&ctg.chr_strand)),
                    ],
                )
                .unwrap();
            let _: () = conn
                .hset_multiple(
                    black_box(&ctg_id),
                    &[
                        ("chr_start", black_box(&ctg.chr_start)),
                        ("chr_end", black_box(&ctg.chr_end)),
                        ("length", black_box(&ctg.length)),
                    ],
                )
                .unwrap();
        })
    });
}

pub fn bench_redis_pipe(c: &mut Criterion) {
    let mut conn = gams::connect();
    let ctg = rand_ctg();

    gams::db_drop();
    c.bench_function("redis_hset_pipe", |b| {
        b.iter(|| {
            let ctg_id = rand_str(4);
            let _: () = redis::pipe()
                .hset(black_box(&ctg_id), "range", black_box(&ctg.range))
                .ignore()
                .hset(black_box(&ctg_id), "chr_id", black_box(&ctg.chr_id))
                .ignore()
                .hset(black_box(&ctg_id), "chr_start", black_box(&ctg.chr_start))
                .ignore()
                .hset(black_box(&ctg_id), "chr_end", black_box(&ctg.chr_end))
                .ignore()
                .hset(black_box(&ctg_id), "chr_strand", black_box(&ctg.chr_strand))
                .ignore()
                .hset(black_box(&ctg_id), "length", black_box(&ctg.length))
                .ignore()
                .query(&mut conn)
                .unwrap();
        })
    });

    gams::db_drop();
    c.bench_function("redis_hset_pipe_10", |b| {
        b.iter(|| {
            let mut batch = &mut redis::pipe();
            for _ in 0..10 {
                let ctg_id = rand_str(4);
                batch = batch
                    .hset(black_box(&ctg_id), "range", black_box(&ctg.range))
                    .ignore()
                    .hset(black_box(&ctg_id), "chr_id", black_box(&ctg.chr_id))
                    .ignore()
                    .hset(black_box(&ctg_id), "chr_start", black_box(&ctg.chr_start))
                    .ignore()
                    .hset(black_box(&ctg_id), "chr_end", black_box(&ctg.chr_end))
                    .ignore()
                    .hset(black_box(&ctg_id), "chr_strand", black_box(&ctg.chr_strand))
                    .ignore()
                    .hset(black_box(&ctg_id), "length", black_box(&ctg.length))
                    .ignore();
            }
            let _: () = batch.query(&mut conn).unwrap();
        })
    });
}

criterion_group!(benches, bench_redis_set, bench_redis_pipe);
criterion_main!(benches);
