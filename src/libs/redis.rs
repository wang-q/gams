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

/// raw redis connection
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

/// drop the database
pub fn db_drop() {
    let mut conn = connect();
    let output: String = redis::cmd("FLUSHDB")
        .query(&mut conn)
        .expect("Failed to execute FLUSHDB");
    println!("{}", output);
}

pub struct Conn {
    conn: redis::Connection,
    // pipe keys-values
    inputs: Vec<(String, String)>,
    // pipe size
    size: usize,
}

/// INTERFACE: Redis connection
/// Three basic data types: str, bin and sn
/// Wrapped data: ctg and seq
///
/// ----
/// ----
impl Conn {
    pub fn new() -> Self {
        Self {
            conn: connect(),
            inputs: vec![],
            size: 0,
        }
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            conn: connect(),
            inputs: vec![],
            size,
        }
    }

    /// raw redis connection
    pub fn conn(&mut self) -> &mut redis::Connection {
        &mut self.conn
    }

    pub fn insert_str(&mut self, key: &str, val: &str) {
        self.conn().set(key, val).unwrap()
    }

    pub fn get_str(&mut self, key: &str) -> String {
        self.conn().get(key).unwrap()
    }

    pub fn insert_bin(&mut self, key: &str, val: &[u8]) {
        self.conn().set(key, val).unwrap()
    }

    pub fn get_bin(&mut self, key: &str) -> Vec<u8> {
        self.conn().get(key).unwrap()
    }

    pub fn incr_sn_n(&mut self, key: &str, n: i32) -> i32 {
        let sn: isize = self.conn().incr(key, n).unwrap();
        sn as i32
    }

    pub fn incr_sn(&mut self, key: &str) -> i32 {
        self.incr_sn_n(key, 1)
    }

    pub fn get_sn(&mut self, key: &str) -> i32 {
        self.conn().get(key).unwrap_or(0)
    }

    pub fn insert_ctg(&mut self, ctg_id: &str, ctg: &crate::Ctg) {
        let json = serde_json::to_string(ctg).unwrap();
        self.insert_str(ctg_id, &json);
    }

    pub fn get_ctg(&mut self, ctg_id: &str) -> crate::Ctg {
        let json = self.get_str(ctg_id);
        serde_json::from_str(&json).unwrap()
    }

    pub fn insert_seq(&mut self, ctg_id: &str, seq: &[u8]) {
        let seq_bytes = encode_gz(seq).unwrap();
        self.insert_bin(&format!("seq:{ctg_id}"), &seq_bytes)
    }

    pub fn get_seq(&mut self, ctg_id: &str) -> String {
        let seq_bytes: Vec<u8> = self.get_bin(&format!("seq:{}", ctg_id));
        let s = decode_gz(&seq_bytes).unwrap();
        String::from_utf8(s).unwrap()
    }
}

fn encode_gz(seq: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut z = flate2::read::GzEncoder::new(seq, flate2::Compression::fast());
    z.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn decode_gz(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut gz = GzDecoder::new(bytes);
    let mut buf = Vec::new();
    gz.read_to_end(&mut buf)?;
    Ok(buf)
}

/// INTERFACE: easy access and index
///
/// ----
/// ----
impl Conn {
    /// get all chr_ids
    pub fn get_vec_chr(&mut self) -> Vec<String> {
        let json = self.get_str("top:chrs");
        serde_json::from_str(&json).unwrap()
    }

    /// generated from cnt:ctg:
    pub fn get_vec_ctg(&mut self, chr_id: &str) -> Vec<String> {
        let key = format!("cnt:ctg:{}", chr_id);
        let cnt = self.get_sn(&key);

        let list: Vec<String> = if cnt == 0 {
            vec![]
        } else {
            (1..=cnt).map(|i| format!("ctg:{}:{}", chr_id, i)).collect()
        };

        list
    }

    /// generated from cnt:
    pub fn get_vec_cnt(&mut self, group: &str, parent_id: &str) -> Vec<String> {
        match group {
            "ctg" => {}
            "feature" => {}
            "rg" => {}
            "peak" => {}
            _ => unreachable!(),
        }

        let cnt_key = format!("cnt:{group}:{parent_id}");
        let cnt = self.get_sn(&cnt_key);

        if cnt == 0 {
            vec![]
        } else {
            (1..=cnt)
                .map(|i| format!("{group}:{parent_id}:{i}"))
                .collect()
        }
    }

    pub fn get_ctg_pos(&mut self, ctg_id: &str) -> (String, i32, i32) {
        let ctg = self.get_ctg(ctg_id);
        (ctg.chr_id, ctg.chr_start, ctg.chr_end)
    }

    /// BTreeMap<ctg_id, Ctg>
    pub fn get_bundle_ctg(&mut self, chr_id: Option<&str>) -> BTreeMap<String, crate::Ctg> {
        let chrs: Vec<String> = if chr_id.is_some() {
            vec![chr_id.unwrap().to_string()]
        } else {
            self.get_vec_chr()
        };

        let mut ctg_of: BTreeMap<String, crate::Ctg> = BTreeMap::new();

        for chr_id in &chrs {
            let ctgs_bytes: Vec<u8> = self.get_bin(&format!("bundle:ctg:{}", chr_id));
            let ctgs: BTreeMap<String, crate::Ctg> = bincode::deserialize(&ctgs_bytes).unwrap();

            ctg_of.extend(ctgs);
        }

        ctg_of
    }

    /// This index helps locating to a ctg
    pub fn build_idx_ctg(&mut self) {
        let chrs: Vec<String> = self.get_vec_chr();

        for chr_id in &chrs {
            let ctgs = self.get_vec_ctg(chr_id);
            let mut ivs: Vec<Iv> = vec![];

            for ctg_id in &ctgs {
                let (_, chr_start, chr_end) = self.get_ctg_pos(ctg_id);
                let iv = Iv {
                    start: chr_start as u32,
                    stop: chr_end as u32 + 1,
                    val: ctg_id.to_string(),
                };
                ivs.push(iv);
            }

            let lapper = Lapper::new(ivs);
            let serialized = bincode::serialize(&lapper).unwrap();

            self.insert_bin(&format!("idx:ctg:{chr_id}"), &serialized);
        }
    }

    /// chr_id => Lapper => ctg_id
    pub fn get_idx_ctg(&mut self) -> BTreeMap<String, Lapper<u32, String>> {
        let mut lapper_of: BTreeMap<String, Lapper<u32, String>> = BTreeMap::new();

        let chrs: Vec<String> = self.get_vec_chr();
        for chr_id in &chrs {
            let bytes: Vec<u8> = self.get_bin(&format!("idx:ctg:{}", chr_id));
            let lapper: Lapper<u32, String> = bincode::deserialize(&bytes).unwrap();

            lapper_of.insert(chr_id.clone(), lapper);
        }

        lapper_of
    }

    /// This index helps counting overlaps
    pub fn build_idx_rg(&mut self) {
        let chrs: Vec<String> = self.get_vec_chr();
        for chr_id in chrs.iter() {
            let ctgs: Vec<String> = self.get_vec_cnt("ctg", chr_id);

            for ctg_id in &ctgs {
                let jsons: Vec<String> = self.get_scan_values(&format!("rg:{}:*", ctg_id));
                let rgs: Vec<crate::Rg> = jsons
                    .iter()
                    .map(|el| serde_json::from_str(el).unwrap())
                    .collect();

                let mut ivs: Vec<Iv> = vec![];
                for rg in &rgs {
                    let range = intspan::Range::from_str(&rg.range);
                    let iv = Iv {
                        start: range.start as u32,
                        stop: range.end as u32 + 1,
                        val: "".to_string(), // we don't need find rg
                    };
                    ivs.push(iv);
                }

                let lapper = Lapper::new(ivs);
                let serialized = bincode::serialize(&lapper).unwrap();

                self.insert_bin(&format!("idx:rg:{ctg_id}"), &serialized);
            }
        }
    }

    /// ctg_id => Lapper
    pub fn get_idx_rg(&mut self) -> BTreeMap<String, Lapper<u32, String>> {
        let mut lapper_of: BTreeMap<String, Lapper<u32, String>> = BTreeMap::new();

        let chrs: Vec<String> = self.get_vec_chr();
        for chr_id in &chrs {
            let ctgs: Vec<String> = self.get_vec_cnt("ctg", chr_id);

            for ctg_id in &ctgs {
                let bytes: Vec<u8> = self.get_bin(&format!("idx:rg:{}", ctg_id));
                let lapper: Lapper<u32, String> = bincode::deserialize(&bytes).unwrap();

                lapper_of.insert(chr_id.clone(), lapper);
            }
        }

        lapper_of
    }
}

/// INTERFACE: lua scripting and pipeline
///
/// ----
/// ----
impl Conn {
    pub fn get_scan_count(&mut self, pattern: &str) -> i32 {
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
        script.arg(pattern).arg(1000).invoke(self.conn()).unwrap()
    }

    pub fn get_scan_keys(&mut self, pattern: &str) -> Vec<String> {
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
        script.arg(pattern).arg(1000).invoke(self.conn()).unwrap()
    }

    pub fn get_scan_values(&mut self, pattern: &str) -> Vec<String> {
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
        script.arg(pattern).arg(1000).invoke(self.conn()).unwrap()
    }

    pub fn pipe_add(&mut self, key: &str, val: &str) {
        self.inputs.push((key.into(), val.into()));

        if self.inputs.len() > self.size {
            self.pipe_submit();
        }
    }

    pub fn pipe_submit(&mut self) {
        if self.inputs.is_empty() {
            return;
        }

        let mut pipe = &mut redis::pipe();

        for (key, val) in self.inputs.iter() {
            pipe = pipe.set(key, val).ignore();
        }
        self.inputs = vec![];

        let _: () = pipe.query(self.conn()).unwrap();
        pipe.clear();
    }
}
