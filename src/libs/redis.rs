use flate2::read::GzDecoder;
use std::collections::BTreeMap;
use std::io::Read;

use redis::Commands;
use serde::Deserialize;

use rust_lapper::{Interval, Lapper};

// Interval: represent a range from [start, stop), carrying val
type Iv = Interval<u32, String>; // the first type should be Unsigned

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

pub fn connect() -> redis::Connection {
    dotenvy::from_filename("gams.env").expect("Failed to read gams.env file");

    let redis_host = dotenvy::var("REDIS_HOST").unwrap();
    let redis_port = dotenvy::var("REDIS_PORT").unwrap();

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

pub fn insert_str(conn: &mut redis::Connection, key: &str, val: &str) {
    conn.set(key, val).unwrap()
}

pub fn get_str(conn: &mut redis::Connection, key: &str) -> String {
    conn.get(key).unwrap()
}

pub fn insert_bin(conn: &mut redis::Connection, key: &str, val: &[u8]) {
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

pub fn insert_ctg(conn: &mut redis::Connection, ctg_id: &str, ctg: &crate::Ctg) {
    let json = serde_json::to_string(ctg).unwrap();
    insert_str(conn, ctg_id, &json);
}

pub fn get_ctg(conn: &mut redis::Connection, ctg_id: &str) -> crate::Ctg {
    let json = get_str(conn, ctg_id);
    serde_json::from_str(&json).unwrap()
}

pub fn get_ctg_pos(conn: &mut redis::Connection, ctg_id: &str) -> (String, i32, i32) {
    let ctg = get_ctg(conn, ctg_id);
    (ctg.chr_id, ctg.chr_start, ctg.chr_end)
}

/// get all chr_ids
pub fn get_vec_chr(conn: &mut redis::Connection) -> Vec<String> {
    let json = get_str(conn, "top:chrs");
    serde_json::from_str(&json).unwrap()
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

        insert_bin(conn, &format!("idx:ctg:{}", chr_id), &serialized);
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
pub fn get_bundle_ctg(
    conn: &mut redis::Connection,
    chr_id: Option<&str>,
) -> BTreeMap<String, crate::Ctg> {
    let chrs: Vec<String> = if chr_id.is_some() {
        vec![chr_id.unwrap().to_string()]
    } else {
        get_vec_chr(conn)
    };

    let mut ctg_of: BTreeMap<String, crate::Ctg> = BTreeMap::new();

    for chr_id in &chrs {
        let ctgs_bytes: Vec<u8> = conn.get(format!("bundle:ctg:{}", chr_id)).unwrap();
        let ctgs: BTreeMap<String, crate::Ctg> = bincode::deserialize(&ctgs_bytes).unwrap();

        ctg_of.extend(ctgs);
    }

    ctg_of
}

pub fn get_scan_count(conn: &mut redis::Connection, pattern: &str) -> i32 {
    let script = redis::Script::new(
        r###"
local cursor = "0";
local count = "0";
repeat
    local result = redis.call('SCAN', cursor, 'MATCH', ARGV[1], 'COUNT', ARGV[2])
    cursor = result[1];
    local count_delta = #result[2];
    count = count + count_delta;
until cursor == "0";
return count;
"###,
    );
    script.arg(pattern).arg(1000).invoke(conn).unwrap()
}

pub fn get_scan_keys(conn: &mut redis::Connection, pattern: &str) -> Vec<String> {
    let script = redis::Script::new(
        r###"
local cursor = "0";
local list = {};
repeat
    local result = redis.call('SCAN', cursor, 'MATCH', ARGV[1], 'COUNT', ARGV[2])
    cursor = result[1];
    for _, key in ipairs(result[2]) do
        list[#list+1] = key
    end
until cursor == "0";
return list;
"###,
    );
    script.arg(pattern).arg(1000).invoke(conn).unwrap()
}

pub fn get_scan_values(conn: &mut redis::Connection, pattern: &str) -> Vec<String> {
    let script = redis::Script::new(
        r###"
local cursor = "0";
local list = {};
repeat
    local result = redis.call('SCAN', cursor, 'MATCH', ARGV[1], 'COUNT', ARGV[2])
    cursor = result[1];
    for _, key in ipairs(result[2]) do
        list[#list+1] = redis.call('GET', key)
    end
until cursor == "0";
return list;
"###,
    );
    script.arg(pattern).arg(1000).invoke(conn).unwrap()
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

/// Don't use this
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

/// Don't use this
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
