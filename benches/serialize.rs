use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::distributions::{Alphanumeric, DistString};
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

pub fn rand_things(c: &mut Criterion) {
    c.bench_function("rand_str", |b| b.iter(|| rand_str(black_box(16))));
    c.bench_function("rand_ctg", |b| b.iter(|| rand_ctg()));
}

pub fn bench_bincode(c: &mut Criterion) {
    let ctg = rand_ctg();
    let bytes = bincode::serialize(&ctg).unwrap();

    c.bench_function("bincode_se_fix", |b| {
        b.iter(|| {
            bincode::serialize(black_box(&ctg)).unwrap();
        })
    });
    c.bench_function("bincode_se_rand", |b| {
        b.iter(|| {
            bincode::serialize(black_box(&rand_ctg())).unwrap();
        })
    });
    c.bench_function("bincode_de", |b| {
        b.iter(|| {
            let _: Ctg = bincode::deserialize(black_box(&bytes)).unwrap();
        })
    });
}

criterion_group!(benches, rand_things, bench_bincode);
criterion_main!(benches);
