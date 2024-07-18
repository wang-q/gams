use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, Read};

use redis::Commands;
use serde::{Deserialize, Serialize};

use rust_lapper::{Interval, Lapper};

// Interval: represent a range from [start, stop), carrying val
type Iv = Interval<u32, String>; // the first type should be Unsigned

use crate::libs::stat::*;
use crate::libs::window::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u32,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u32 {
    6379
}

// ID   range   chr_id	chr_start	chr_end	chr_strand	length
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ctg {
    pub id: String,
    pub range: String,
    pub chr_id: String,
    pub chr_start: i32,
    pub chr_end: i32,
    pub chr_strand: String,
    pub length: i32,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub range: String,
    pub chr_id: String,
    pub chr_start: i32,
    pub chr_end: i32,
    pub chr_strand: String,
    pub length: i32,
    pub ctg_id: String,
    pub tag: String,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rg {
    pub id: String,
    pub range: String,
    pub ctg_id: String,
}

pub fn connect() -> redis::Connection {
    dotenv::from_filename("gams.env").expect("Failed to read gams.env file");

    let redis_host = dotenv::var("REDIS_HOST").unwrap();
    let redis_port = dotenv::var("REDIS_PORT").unwrap();

    // if Redis server needs secure connection
    // let uri_scheme = match redis_tls.as_ref() {
    //     "true" => "rediss",
    //     "false" => "redis",
    //     _ => "redis",
    // };
    let uri_scheme = "redis";

    let redis_conn_url = format!("{}://{}:{}", uri_scheme, redis_host, redis_port);
    //println!("{}", redis_conn_url);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("Failed to connect to Redis")
}

pub fn insert_str(conn: &mut redis::Connection, key: &str, val: &str) -> () {
    conn.set(key, val).unwrap()
}

pub fn get_str(conn: &mut redis::Connection, key: &str) -> String {
    conn.get(key).unwrap()
}

pub fn insert_bin(conn: &mut redis::Connection, key: &str, val: &[u8]) -> () {
    conn.set(key, val).unwrap()
}

pub fn get_bin(conn: &mut redis::Connection, key: &str) -> Vec<u8> {
    conn.get(key).unwrap()
}

pub fn incr_serial(conn: &mut redis::Connection, key: &str) -> isize {
    incr_serial_n(conn, key, 1)
}

pub fn incr_serial_n(conn: &mut redis::Connection, key: &str, n: i32) -> isize {
    conn.incr(key, n).unwrap()
}

fn encode_gz(seq: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut z = flate2::read::GzEncoder::new(seq, flate2::Compression::fast());
    z.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn insert_seq(conn: &mut redis::Connection, ctg_id: &str, seq: &[u8]) {
    let seq_bytes = encode_gz(seq).unwrap();
    insert_bin(conn, &format!("seq:{ctg_id}"), &seq_bytes);
}

fn decode_gz(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut gz = GzDecoder::new(bytes);
    let mut buf = Vec::new();
    gz.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn get_seq(conn: &mut redis::Connection, ctg_id: &str) -> String {
    let seq_bytes: Vec<u8> = conn.get(format!("seq:{}", ctg_id)).unwrap();

    let s = decode_gz(&seq_bytes).unwrap();
    String::from_utf8(s).unwrap()
}

pub fn insert_ctg(conn: &mut redis::Connection, ctg_id: &str, ctg: &Ctg) {
    let ctg_bytes = bincode::serialize(ctg).unwrap();
    insert_bin(conn, ctg_id, &ctg_bytes);
}

pub fn get_ctg(conn: &mut redis::Connection, ctg_id: &str) -> Ctg {
    let ctg_bytes = get_bin(conn, ctg_id);
    bincode::deserialize(&ctg_bytes).unwrap()
}

pub fn get_ctg_pos(conn: &mut redis::Connection, ctg_id: &str) -> (String, i32, i32) {
    let ctg = get_ctg(conn, ctg_id);
    (ctg.chr_id, ctg.chr_start, ctg.chr_end)
}

/// get all chr_ids
pub fn get_vec_chr(conn: &mut redis::Connection) -> Vec<String> {
    let bin = get_bin(conn, "top:chrs");
    bincode::deserialize(&*bin).unwrap()
}

/// generated from cnt:ctg:
pub fn get_vec_ctg(conn: &mut redis::Connection, chr_id: &str) -> Vec<String> {
    let cnt = conn.get(format!("cnt:ctg:{}", chr_id)).unwrap_or(0);

    let list: Vec<String> = if cnt == 0 {
        vec![]
    } else {
        (1..=cnt).map(|i| format!("ctg:{}:{}", chr_id, i)).collect()
    };

    list
}

/// generated from cnt:feature:
pub fn get_vec_feature(conn: &mut redis::Connection, ctg_id: &str) -> Vec<String> {
    let cnt = conn.get(format!("cnt:feature:{}", ctg_id)).unwrap_or(0);

    let list: Vec<String> = if cnt == 0 {
        vec![]
    } else {
        (1..=cnt)
            .map(|i| format!("feature:{}:{}", ctg_id, i))
            .collect()
    };

    list
}

/// generated from cnt:peak:
pub fn get_vec_peak(conn: &mut redis::Connection, ctg_id: &str) -> Vec<String> {
    let cnt = conn.get(format!("cnt:peak:{}", ctg_id)).unwrap_or(0);

    let list: Vec<String> = if cnt == 0 {
        vec![]
    } else {
        (1..=cnt)
            .map(|i| format!("peak:{}:{}", ctg_id, i))
            .collect()
    };

    list
}

pub fn build_idx_ctg(conn: &mut redis::Connection) {
    let chrs: Vec<String> = get_vec_chr(conn);

    for chr_id in &chrs {
        let ctgs = get_vec_ctg(conn, chr_id);
        let mut ivs: Vec<Iv> = vec![];

        for ctg_id in &ctgs {
            let (_, chr_start, chr_end) = get_ctg_pos(conn, ctg_id);
            let iv = Iv {
                start: chr_start as u32,
                stop: chr_end as u32 + 1,
                val: ctg_id.to_string(),
            };
            ivs.push(iv);
        }

        let lapper = Lapper::new(ivs);
        let serialized = bincode::serialize(&lapper).unwrap();

        let _: () = conn
            .set(format!("idx:ctg:{}", chr_id), &serialized)
            .unwrap();
    }
}

/// chr_id => Lapper => ctg_id
pub fn get_idx_ctg(conn: &mut redis::Connection) -> BTreeMap<String, Lapper<u32, String>> {
    let chrs: Vec<String> = get_vec_chr(conn);

    let mut lapper_of: BTreeMap<String, Lapper<u32, String>> = BTreeMap::new();

    for chr_id in &chrs {
        let lapper_bytes: Vec<u8> = conn.get(format!("idx:ctg:{}", chr_id)).unwrap();
        let lapper: Lapper<u32, String> = bincode::deserialize(&lapper_bytes).unwrap();

        lapper_of.insert(chr_id.clone(), lapper);
    }

    lapper_of
}

/// BTreeMap<ctg_id, Ctg>
pub fn get_bin_ctgs(conn: &mut redis::Connection) -> BTreeMap<String, Ctg> {
    let chrs: Vec<String> = get_vec_chr(conn);

    let mut ctg_of: BTreeMap<String, Ctg> = BTreeMap::new();

    for chr_id in &chrs {
        let ctgs_bytes: Vec<u8> = conn.get(format!("bin:ctg:{}", chr_id)).unwrap();
        let ctgs: BTreeMap<String, Ctg> = bincode::deserialize(&ctgs_bytes).unwrap();

        ctg_of.extend(ctgs);
    }

    ctg_of
}

/// bincode stored in a Redis set
pub fn get_bin_features(conn: &mut redis::Connection, ctg_id: &str) -> Vec<Feature> {
    let features_bytes: Vec<Vec<u8>> = conn
        .smembers(format!("bin:feature:{}", ctg_id))
        .unwrap_or(vec![]);

    features_bytes
        .iter()
        .map(|el| bincode::deserialize(el).unwrap())
        .collect()
}

// Can't do this
// Response was of incompatible type - TypeError: "Bulk response of wrong dimension"
// pub fn batch_key_pos(conn: &mut redis::Connection, keys: &Vec<String>) -> Vec<(String, i32, i32)> {
//     let mut result: Vec<(String, i32, i32)> = vec![];
//
//     let mut batch = &mut redis::pipe();
//
//     for (i, key) in keys.iter().enumerate() {
//         if i > 1 && i % 100 == 0 {
//             let ary: Vec<(String, i32, i32)> = batch.query(conn).unwrap();
//             result.extend(ary);
//             batch.clear();
//         }
//
//         batch = batch
//             .hget(key, &["chr_id", "chr_start", "chr_end"]);
//     }
//     // Possible remaining records in the batch
//     {
//         let ary: Vec<(String, i32, i32)> = batch.query(conn).unwrap();
//         result.extend(ary);
//         batch.clear();
//     }
//
//     result
// }

pub fn get_scan_count(conn: &mut redis::Connection, pattern: &str) -> i32 {
    // number of matches
    let mut count = 0;
    let iter: redis::Iter<'_, String> = redis::cmd("SCAN")
        .cursor_arg(0)
        .arg("MATCH")
        .arg(pattern)
        .arg("COUNT")
        .arg(1000) // default is 10
        .clone()
        .iter(conn)
        .unwrap();
    for _ in iter {
        count += 1;
    }

    count
}

///
/// ```
/// // let mut conn = gams::connect();
///
/// // let keys = gams::get_scan_vec(&mut conn, "prefix:*");
/// ```
pub fn get_scan_vec_n(conn: &mut redis::Connection, pattern: &str, count: usize) -> Vec<String> {
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

pub fn get_scan_vec(conn: &mut redis::Connection, pattern: &str) -> Vec<String> {
    get_scan_vec_n(conn, pattern, 1000)
}

/// Default COUNT is 10
pub fn get_scan_match_vec(conn: &mut redis::Connection, scan: &str) -> Vec<String> {
    // number of matches
    let mut keys: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match(scan).unwrap();
    for x in iter {
        keys.push(x);
    }

    keys
}

pub fn find_one_idx(
    lapper_of: &BTreeMap<String, Lapper<u32, String>>,
    rg: &intspan::Range,
) -> String {
    if !lapper_of.contains_key(rg.chr()) {
        return "".to_string();
    }

    let lapper = lapper_of.get(rg.chr()).unwrap();
    let res = lapper.find(*rg.start() as u32, *rg.end() as u32).next();

    match res {
        Some(iv) => iv.val.clone(),
        None => "".to_string(),
    }
}

pub fn find_one_l(conn: &mut redis::Connection, rg: &intspan::Range) -> String {
    let lapper_bytes: Vec<u8> = conn.get(format!("idx:ctg:{}", rg.chr())).unwrap();

    if lapper_bytes.is_empty() {
        return "".to_string();
    }

    let lapper: Lapper<u32, String> = bincode::deserialize(&lapper_bytes).unwrap();
    let res = lapper.find(*rg.start() as u32, *rg.end() as u32).next();

    match res {
        Some(iv) => iv.val.clone(),
        None => "".to_string(),
    }
}

/// GC-content within a ctg
pub fn cache_gc_content(
    rg: &intspan::Range,
    parent: &intspan::IntSpan,
    seq: &str,
    cache: &mut HashMap<String, f32>,
) -> f32 {
    let field = rg.to_string();

    if !cache.contains_key(&field) {
        // converted to ctg index
        let from = parent.index(*rg.start()) as usize;
        let to = parent.index(*rg.end()) as usize;

        // from <= x < to, zero-based
        let sub_seq = seq.get((from - 1)..(to)).unwrap().bytes();

        let gc_content = bio::seq_analysis::gc::gc_content(sub_seq);
        cache.insert(field.clone(), gc_content);
    };

    *cache.get(&field).unwrap()
}

pub fn cache_gc_stat(
    rg: &intspan::Range,
    parent: &intspan::IntSpan,
    seq: &str,
    cache: &mut HashMap<String, f32>,
    size: i32,
    step: i32,
) -> (f32, f32, f32) {
    let intspan = rg.intspan();
    let windows = sliding(&intspan, size, step);

    let mut gcs = Vec::new();

    for w in windows {
        let gc_content = cache_gc_content(
            &intspan::Range::from(rg.chr(), w.min(), w.max()),
            parent,
            seq,
            cache,
        );
        gcs.push(gc_content);
    }

    gc_stat(&gcs)
}

pub fn get_rg_seq(conn: &mut redis::Connection, rg: &intspan::Range) -> String {
    let ctg_id = find_one_l(conn, rg);

    if ctg_id.is_empty() {
        return "".to_string();
    }

    let ctg = get_ctg(conn, &ctg_id);
    let chr_start = ctg.chr_start;

    let ctg_start = (rg.start() - chr_start + 1) as usize;
    let ctg_end = (rg.end() - chr_start + 1) as usize;

    let ctg_seq = get_seq(conn, &ctg_id);

    // from <= x < to, zero-based
    let seq = ctg_seq.get((ctg_start - 1)..(ctg_end)).unwrap();

    String::from(seq)
}

/// Read ranges in the file
pub fn read_range(
    infile: &str,
    lapper_of: &BTreeMap<String, Lapper<u32, String>>,
) -> BTreeMap<String, Vec<intspan::Range>> {
    let reader = intspan::reader(infile);

    // ctg_id => [Range]
    let mut ranges_of: BTreeMap<String, Vec<intspan::Range>> = BTreeMap::new();

    // processing each line
    for line in reader.lines().map_while(Result::ok) {
        let rg = intspan::Range::from_str(&line);
        if !rg.is_valid() {
            continue;
        }

        let ctg_id = find_one_idx(lapper_of, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        ranges_of
            .entry(ctg_id)
            .and_modify(|v| v.push(rg))
            .or_default();
    }

    ranges_of
}

/// This is an expensive operation
pub fn get_gc_content(conn: &mut redis::Connection, rg: &intspan::Range) -> f32 {
    let bucket = format!("cache:{}:{}", rg.chr(), rg.start() / 1000);
    let field = rg.to_string();
    let expire = 180;
    let res = conn.hget(&bucket, &field).unwrap();

    return match res {
        Some(res) => {
            let _: () = conn.expire(&bucket, expire).unwrap();

            res
        }
        None => {
            let seq = get_rg_seq(conn, rg);

            let gc_content = if seq.is_empty() {
                0.
            } else {
                bio::seq_analysis::gc::gc_content(seq.bytes())
            };
            let _: () = conn.hset(&bucket, &field, gc_content).unwrap();
            let _: () = conn.expire(&bucket, expire).unwrap();

            gc_content
        }
    };
}

/// drop the database
pub fn db_drop() {
    let mut conn = connect();
    let output: String = redis::cmd("FLUSHDB")
        .query(&mut conn)
        .expect("Failed to execute FLUSHDB");
    println!("{}", output);
}

pub fn extract_ctg_id(input: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?xi)
            (?P<ctg>ctg:[\w_]+:\d+)
            "
        )
        .unwrap();
    }
    RE.captures(input)
        .and_then(|cap| cap.name("ctg").map(|ctg| ctg.as_str()))
}
