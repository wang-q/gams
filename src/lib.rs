use intspan::{IntSpan, Range};
use redis::Commands;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

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

pub fn find_one(conn: &mut redis::Connection, chr: &str, start: i32, end: i32) -> String {
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
        .arg(format!("tmp-s:{}", chr))
        .arg(format!("ctg-s:{}", chr))
        .arg(0)
        .arg(start)
        .arg("BYSCORE")
        .ignore()
        .cmd("ZRANGESTORE")
        .arg(format!("tmp-e:{}", chr))
        .arg(format!("ctg-e:{}", chr))
        .arg(end)
        .arg("+inf")
        .arg("BYSCORE")
        .ignore()
        .zinterstore_min(
            format!("tmp-ctg:{}", chr),
            &[format!("tmp-s:{}", chr), format!("tmp-e:{}", chr)],
        )
        .ignore()
        .del(format!("tmp-s:{}", chr))
        .ignore()
        .del(format!("tmp-e:{}", chr))
        .ignore()
        .cmd("ZPOPMIN")
        .arg(format!("tmp-ctg:{}", chr))
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

pub fn get_seq(conn: &mut redis::Connection, chr: &str, start: i32, end: i32) -> String {
    let ctg = find_one(conn, chr, start, end);

    if ctg.is_empty() {
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

// TODO: caches of gc_content
pub fn get_gc_content(conn: &mut redis::Connection, chr: &str, start: i32, end: i32) -> f32 {
    let seq = get_seq(conn, chr, start, end);

    if seq.is_empty() {
        return 0 as f32;
    }

    bio::seq_analysis::gc::gc_content(seq.bytes())
}

pub fn get_gc_stat(
    conn: &mut redis::Connection,
    r: &str,
    size: i32,
    step: i32,
) -> (f32, f32, f32, f32) {
    let range = Range::from_str(r);

    let intspan = IntSpan::from_pair(*range.start(), *range.end());
    let windows = sliding(&intspan, size, step);

    let mut gcs = Vec::new();

    for w in windows {
        let gc_content = get_gc_content(conn, range.chr(), w.min(), w.max());
        gcs.push(gc_content);
    }

    gc_stat(&gcs)
}

pub fn gc_stat(gcs: &Vec<f32>) -> (f32, f32, f32, f32) {
    let mean = mean(gcs);
    let stddev = stddev(gcs);

    // coefficient of variation
    let cv = if mean == 0. || mean == 1. {
        0.
    } else if mean <= 0.5 {
        stddev / mean
    } else {
        stddev / (1. - mean)
    };

    // Signal-to-noise ratio
    let snr = if stddev == 0. {
        0.
    } else if mean <= 0.5 {
        mean / stddev
    } else {
        (1. - mean) / stddev
    };

    (mean, stddev, cv, snr)
}

pub fn mean(data: &[f32]) -> f32 {
    let len = data.len() as f32;
    let sum = data.iter().sum::<f32>();

    sum / len
}

pub fn stddev(data: &[f32]) -> f32 {
    let len = data.len() as f32;
    let mean = mean(data);

    let sq_sum = data.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>();
    (sq_sum / (len - 1.)).sqrt()
}

pub fn sliding(intspan: &IntSpan, size: i32, step: i32) -> Vec<IntSpan> {
    let mut windows = vec![];

    let mut start = 1;
    loop {
        let end = start + size - 1;
        if end > intspan.size() {
            break;
        }
        let window = intspan.slice(start, end);
        start += step;

        windows.push(window);
    }

    windows
}

pub fn thresholding_algo(data: &Vec<f32>, lag: usize, threshold: f32, influence: f32) -> Vec<i32> {
    //  the results (peaks, 1 or -1)
    let mut signals: Vec<i32> = vec![0; data.len()];

    // filter out the signals (peaks) from original list (using influence arg)
    let mut filtered_data: Vec<f32> = data.clone();

    // the current average of the rolling window
    let mut avg_filter: Vec<f32> = vec![0.; data.len()];

    // the current standard deviation of the rolling window
    let mut std_filter: Vec<f32> = vec![0.; data.len()];

    // init avg_filter & std_filter
    avg_filter[lag - 1] = mean(&data[0..lag]);
    std_filter[lag - 1] = stddev(&data[0..lag]);

    // loop input starting at end of rolling window
    for i in lag..data.len() {
        // if the distance between the current value and average is enough standard deviations (threshold) away
        if (data[i] - avg_filter[i - 1]).abs() > threshold * std_filter[i - 1] {
            // this is a signal (i.e. peak), determine if it is a positive or negative signal
            signals[i] = if data[i] > avg_filter[i - 1] { 1 } else { -1 };

            // filter this signal out using influence
            // $filteredY[$i] = $influence * $y->[$i] + (1 - $influence) * $filteredY[$i-1];
            filtered_data[i] = influence * data[i] + (1. - influence) * filtered_data[i - 1];
        } else {
            // ensure this signal remains a zero
            signals[i] = 0;
            // ensure this value is not filtered
            filtered_data[i] = data[i];
        }

        // update average & deviation
        avg_filter[i] = mean(&filtered_data[(i - lag)..i]);
        std_filter[i] = stddev(&filtered_data[(i - lag)..i]);
    }

    signals
}

#[cfg(test)]
mod tests {
    use crate::thresholding_algo;

    #[test]
    fn sample_data() {
        let input: Vec<f32> = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, //
            1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 1.0, //
            1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, //
            1.1, 1.0, 1.0, 1.1, 1.0, 0.8, 0.9, 1.0, 1.2, 0.9, //
            1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, //
            3.0, 2.0, 1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 3.0, 2.6, //
            4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0, 4.0, 2.0, //
            2.5, 1.0, 1.0, 1.0,
        ];
        let exp = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 1, 0, 1, 1, 1, //
            1, 1, 0, 0, 0, 0, 0, 0, 1, 1, //
            1, 1, 1, 1, 0, 0, 0, 1, 1, 1, //
            1, 0, 0, 0,
        ];
        assert_eq!(thresholding_algo(&input, 30, 5., 0.), exp);
    }
}
