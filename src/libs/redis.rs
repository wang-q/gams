use flate2::read::GzDecoder;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::io::Read;

use intspan::{IntSpan, Range};
use redis::Commands;
use serde::Deserialize;

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

pub fn get_key_pos(conn: &mut redis::Connection, key: &str) -> (String, i32, i32) {
    let chr_name: String = conn.hget(key, "chr_name").unwrap();
    let chr_start: i32 = conn.hget(key, "chr_start").unwrap();
    let chr_end: i32 = conn.hget(key, "chr_end").unwrap();

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

pub fn find_one(conn: &mut redis::Connection, rg: &Range) -> String {
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

pub fn get_rg_seq(conn: &mut redis::Connection, rg: &Range) -> String {
    let ctg_id = find_one(conn, rg);

    if ctg_id.is_empty() {
        return "".to_string();
    }

    let chr_start: i32 = conn.hget(&ctg_id, "chr_start").unwrap();

    let ctg_start = (rg.start() - chr_start + 1) as usize;
    let ctg_end = (rg.end() - chr_start + 1) as usize;

    let ctg_seq = get_ctg_seq(conn, &ctg_id);

    let seq: String = String::from(&ctg_seq[ctg_start - 1..ctg_end]);

    seq
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

pub fn ctg_gc_content(
    conn: &mut redis::Connection,
    rg: &Range,
    parent: &IntSpan,
    seq: &String,
) -> f32 {
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
            // converted to ctg index
            let from = parent.index(*rg.start()) as usize;
            let to = parent.index(*rg.end()) as usize;

            // from <= x < to, zero-based
            let sub_seq = seq.get((from - 1)..(to)).unwrap().bytes();
            let gc_content = bio::seq_analysis::gc::gc_content(sub_seq);

            let _: () = conn.hset(&bucket, &field, gc_content).unwrap();
            let _: () = conn.expire(&bucket, expire).unwrap();

            gc_content
        }
    };
}

pub fn get_gc_stat(
    conn: &mut redis::Connection,
    rg: &Range,
    size: i32,
    step: i32,
) -> (f32, f32, f32, f32) {
    let intspan = rg.intspan();
    let windows = sliding(&intspan, size, step);

    let mut gcs = Vec::new();

    for w in windows {
        let gc_content = get_gc_content(conn, &Range::from(rg.chr(), w.min(), w.max()));
        gcs.push(gc_content);
    }

    gc_stat(&gcs)
}

pub fn ctg_gc_stat(
    conn: &mut redis::Connection,
    rg: &Range,
    size: i32,
    step: i32,
    parent: &IntSpan,
    seq: &String,
) -> (f32, f32, f32, f32) {
    let intspan = rg.intspan();
    let windows = sliding(&intspan, size, step);

    let mut gcs = Vec::new();

    for w in windows {
        let gc_content =
            ctg_gc_content(conn, &Range::from(rg.chr(), w.min(), w.max()), parent, seq);
        gcs.push(gc_content);
    }

    gc_stat(&gcs)
}
