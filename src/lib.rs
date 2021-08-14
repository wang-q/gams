use redis::Commands;
use serde::Deserialize;
use std::collections::BTreeMap;

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
    dotenv::from_filename("garr.env").expect("Failed to read garr.env file");

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

pub fn get_ctgs(conn: &mut redis::Connection) -> Vec<String> {
    // number of ctg
    let mut ctgs: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match("ctg:*").unwrap();
    for x in iter {
        ctgs.push(x);
    }

    ctgs
}

pub fn find_one(conn: &mut redis::Connection, name: &str, start: i32, end: i32) -> String {
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
        .arg(format!("tmp-s:{}", name))
        .arg(format!("ctg-s:{}", name))
        .arg(0)
        .arg(start)
        .arg("BYSCORE")
        .ignore()
        .cmd("ZRANGESTORE")
        .arg(format!("tmp-e:{}", name))
        .arg(format!("ctg-e:{}", name))
        .arg(end)
        .arg("+inf")
        .arg("BYSCORE")
        .ignore()
        .zinterstore_min(
            format!("tmp-ctg:{}", name),
            &[format!("tmp-s:{}", name), format!("tmp-e:{}", name)],
        )
        .ignore()
        .del(format!("tmp-s:{}", name))
        .ignore()
        .del(format!("tmp-e:{}", name))
        .ignore()
        .cmd("ZPOPMIN")
        .arg(format!("tmp-ctg:{}", name))
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

pub fn get_seq(conn: &mut redis::Connection, name: &str, start: i32, end: i32) -> String {
    let ctg = find_one(conn, name, start, end);

    if ctg == "" {
        return "".to_string();
    }

    let chr_start: i32 = conn.hget(&ctg, "chr_start").unwrap();

    let ctg_start = (start - chr_start + 1) as isize;
    let ctg_end = (end - chr_start + 1) as isize;

    let seq: String = conn
        .getrange(format!("seq:{}", ctg), ctg_start - 1, ctg_end - 1)
        .unwrap();

    seq
}
