use flate2::read::GzDecoder;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::io::Read;

use intspan::{IntSpan, Range};
use redis::Commands;
use serde::Deserialize;

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
    pub redis_password: Option<String>,
    #[serde(default = "default_redis_tls")]
    pub redis_tls: bool,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u32 {
    6379
}

fn default_redis_tls() -> bool {
    false
}

pub fn connect() -> redis::Connection {
    dotenv::from_filename("gars.env").expect("Failed to read gars.env file");

    let redis_host = dotenv::var("REDIS_HOST").unwrap();
    let redis_port = dotenv::var("REDIS_PORT").unwrap();
    let redis_password = dotenv::var("REDIS_PASSWORD").unwrap_or_default();
    let redis_tls = dotenv::var("REDIS_TLS").unwrap();

    // if Redis server needs secure connection
    let uri_scheme = match redis_tls.as_ref() {
        "true" => "rediss",
        "false" => "redis",
        _ => "redis",
    };

    let redis_conn_url = format!(
        "{}://:{}@{}:{}",
        uri_scheme, redis_password, redis_host, redis_port
    );
    //println!("{}", redis_conn_url);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("Failed to connect to Redis")
}

/// get all chr_ids
pub fn get_vec_chr(conn: &mut redis::Connection) -> Vec<String> {
    conn.hkeys("chr").unwrap()
}

/// generated from cnt:ctg:
pub fn get_vec_ctg(conn: &mut redis::Connection, chr_id: &String) -> Vec<String> {
    let cnt = conn.get(format!("cnt:ctg:{}", chr_id)).unwrap_or(0);

    let list: Vec<String> = if cnt == 0 {
        vec![]
    } else {
        (1..=cnt)
            .into_iter()
            .map(|i| format!("ctg:{}:{}", chr_id, i))
            .collect()
    };

    list
}

pub fn get_key_pos(conn: &mut redis::Connection, key: &str) -> (String, i32, i32) {
    let (chr_name, chr_start, chr_end): (String, i32, i32) = conn
        .hget(key, &["chr_name", "chr_start", "chr_end"])
        .unwrap();

    (chr_name, chr_start, chr_end)
}

pub fn get_scan_count(conn: &mut redis::Connection, scan: String) -> i32 {
    // number of matches
    let mut count = 0;
    let iter: redis::Iter<'_, String> = conn.scan_match(scan).unwrap();
    for _ in iter {
        count += 1;
    }

    count
}

pub fn get_scan_vec(conn: &mut redis::Connection, scan: String) -> Vec<String> {
    // number of matches
    let mut keys: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match(scan).unwrap();
    for x in iter {
        keys.push(x);
    }

    keys
}

pub fn get_scan_str(
    conn: &mut redis::Connection,
    scan: String,
    field: String,
) -> HashMap<String, String> {
    // number of matches
    let keys: Vec<String> = get_scan_vec(conn, scan);

    let mut hash: HashMap<String, _> = HashMap::new();
    for key in keys {
        let val: String = conn.hget(&key, &field).unwrap();
        hash.insert(key.clone(), val);
    }

    hash
}

pub fn get_scan_int(
    conn: &mut redis::Connection,
    scan: String,
    field: String,
) -> HashMap<String, i32> {
    // number of matches
    let keys: Vec<String> = get_scan_vec(conn, scan);

    let mut hash: HashMap<String, _> = HashMap::new();
    for key in keys {
        let val: i32 = conn.hget(&key, &field).unwrap();
        hash.insert(key.clone(), val);
    }

    hash
}

pub fn build_idx_ctg(conn: &mut redis::Connection) {
    // seq_name => Vector of Intervals
    let mut ivs_of: BTreeMap<String, Vec<Iv>> = BTreeMap::new();

    // each ctg
    let ctgs: Vec<String> = get_scan_vec(conn, "ctg:*".to_string());
    for ctg_id in ctgs {
        let (chr_name, chr_start, chr_end) = get_key_pos(conn, &ctg_id);

        if !ivs_of.contains_key(chr_name.as_str()) {
            let ivs: Vec<Iv> = vec![];
            ivs_of.insert(chr_name.clone(), ivs);
        }

        let iv = Iv {
            start: chr_start as u32,
            stop: chr_end as u32 + 1,
            val: ctg_id,
        };

        ivs_of
            .entry(chr_name.to_string())
            .and_modify(|e| e.push(iv));
    }

    for chr in ivs_of.keys() {
        let lapper = Lapper::new(ivs_of.get(chr).unwrap().to_owned());
        let serialized = bincode::serialize(&lapper).unwrap();

        // hash lapper
        let _: () = conn.hset("lapper", chr, &serialized).unwrap();
    }
}

pub fn get_idx_ctg(conn: &mut redis::Connection) -> BTreeMap<String, Lapper<u32, String>> {
    // seq_name => Lapper => ctg_id
    let mut lapper_of: BTreeMap<String, Lapper<u32, String>> = BTreeMap::new();

    let chrs: Vec<String> = conn.hkeys("lapper").unwrap();

    for chr in chrs {
        let lapper_bytes: Vec<u8> = conn.hget("lapper", chr.as_str()).unwrap();
        let lapper: Lapper<u32, String> = bincode::deserialize(&lapper_bytes).unwrap();

        lapper_of.insert(chr.clone(), lapper);
    }

    lapper_of
}

pub fn find_one_idx(lapper_of: &BTreeMap<String, Lapper<u32, String>>, rg: &Range) -> String {
    if !lapper_of.contains_key(rg.chr()) {
        return "".to_string();
    }

    let lapper = lapper_of.get(rg.chr()).unwrap();
    let res = lapper.find(*rg.start() as u32, *rg.end() as u32).next();

    return match res {
        Some(iv) => iv.val.clone(),
        None => "".to_string(),
    };
}

pub fn find_one_l(conn: &mut redis::Connection, rg: &Range) -> String {
    let lapper_bytes: Vec<u8> = conn.hget("lapper", rg.chr()).unwrap();

    if lapper_bytes.is_empty() {
        return "".to_string();
    }

    let lapper: Lapper<u32, String> = bincode::deserialize(&lapper_bytes).unwrap();
    let res = lapper.find(*rg.start() as u32, *rg.end() as u32).next();

    return match res {
        Some(iv) => iv.val.clone(),
        None => "".to_string(),
    };
}

pub fn find_one_z(conn: &mut redis::Connection, rg: &Range) -> String {
    // MULTI
    // ZRANGESTORE tmp-s:I ctg-s:I 0 1000 BYSCORE
    // ZRANGESTORE tmp-e:I ctg-e:I 1100 +inf BYSCORE
    // ZINTERSTORE tmp-ctg:I 2 tmp-s:I tmp-e:I AGGREGATE MIN
    // DEL tmp-s:I tmp-e:I
    // ZPOPMIN tmp-ctg:I
    // EXEC

    let res: Vec<BTreeMap<String, isize>> = redis::pipe()
        .atomic()
        .cmd("ZRANGESTORE")
        .arg(format!("tmp-s:{}", rg.chr()))
        .arg(format!("ctg-s:{}", rg.chr()))
        .arg(0)
        .arg(*rg.start())
        .arg("BYSCORE")
        .ignore()
        .cmd("ZRANGESTORE")
        .arg(format!("tmp-e:{}", rg.chr()))
        .arg(format!("ctg-e:{}", rg.chr()))
        .arg(*rg.end())
        .arg("+inf")
        .arg("BYSCORE")
        .ignore()
        .zinterstore_min(
            format!("tmp-ctg:{}", rg.chr()),
            &[format!("tmp-s:{}", rg.chr()), format!("tmp-e:{}", rg.chr())],
        )
        .ignore()
        .del(format!("tmp-s:{}", rg.chr()))
        .ignore()
        .del(format!("tmp-e:{}", rg.chr()))
        .ignore()
        .cmd("ZPOPMIN")
        .arg(format!("tmp-ctg:{}", rg.chr()))
        .arg(1)
        .query(conn)
        .unwrap();

    // res = [
    //     {
    //         "ctg:I:1": 1,
    //     },
    // ]
    let first = res.first().unwrap();
    let key = match first.iter().next() {
        Some(i) => i.0,
        _ => "",
    };

    key.to_string()
}

pub fn decode_gz(bytes: &[u8]) -> io::Result<String> {
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut s = String::new();
    gz.read_to_string(&mut s)?;
    Ok(s)
}

pub fn get_ctg_seq(conn: &mut redis::Connection, ctg_id: &str) -> String {
    let ctg_bytes: Vec<u8> = conn.get(format!("seq:{}", ctg_id)).unwrap();
    let ctg_seq = decode_gz(&ctg_bytes).unwrap();

    ctg_seq
}

/// GC-content within a ctg
pub fn cache_gc_content(
    rg: &Range,
    parent: &IntSpan,
    seq: &String,
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
    rg: &Range,
    parent: &IntSpan,
    seq: &String,
    cache: &mut HashMap<String, f32>,
    size: i32,
    step: i32,
) -> (f32, f32, f32, f32) {
    let intspan = rg.intspan();
    let windows = sliding(&intspan, size, step);

    let mut gcs = Vec::new();

    for w in windows {
        let gc_content =
            cache_gc_content(&Range::from(rg.chr(), w.min(), w.max()), parent, seq, cache);
        gcs.push(gc_content);
    }

    gc_stat(&gcs)
}

pub fn get_rg_seq(conn: &mut redis::Connection, rg: &Range) -> String {
    let ctg_id = find_one_l(conn, rg);

    if ctg_id.is_empty() {
        return "".to_string();
    }

    let chr_start: i32 = conn.hget(&ctg_id, "chr_start").unwrap();

    let ctg_start = (rg.start() - chr_start + 1) as usize;
    let ctg_end = (rg.end() - chr_start + 1) as usize;

    let ctg_seq = get_ctg_seq(conn, &ctg_id);

    // from <= x < to, zero-based
    let seq = ctg_seq.get((ctg_start - 1)..(ctg_end)).unwrap();

    String::from(seq)
}

/// This is an expensive operation
pub fn get_gc_content(conn: &mut redis::Connection, rg: &Range) -> f32 {
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
