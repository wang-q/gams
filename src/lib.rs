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

pub fn get_seq(conn: &mut redis::Connection, rg: &Range) -> String {
    let ctg_id = find_one(conn, rg);

    if ctg_id.is_empty() {
        return "".to_string();
    }

    let chr_start: i32 = conn.hget(&ctg_id, "chr_start").unwrap();

    let ctg_start = (rg.start() - chr_start + 1) as isize;
    let ctg_end = (rg.end() - chr_start + 1) as isize;

    let seq: String = conn
        .getrange(format!("seq:{}", ctg_id), ctg_start - 1, ctg_end - 1)
        .unwrap();

    seq
}

pub fn get_gc_content(conn: &mut redis::Connection, rg: &Range) -> f32 {
    let key = format!("cache:{}", rg.to_string());
    let res = conn.get(&key).unwrap();

    return match res {
        Some(res) => res,
        None => {
            let seq = get_seq(conn, rg);

            let gc_content = if seq.is_empty() {
                0.
            } else {
                bio::seq_analysis::gc::gc_content(seq.bytes())
            };
            let _: () = conn.set(&key, gc_content).unwrap();

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

pub fn center_sw(
    parent: &IntSpan,
    start: i32,
    end: i32,
    size: i32,
    max: i32,
) -> Vec<(IntSpan, String, i32)> {
    let mut windows = vec![];

    let w0 = center_resize(parent, &IntSpan::from_pair(start, end), size);
    windows.push((w0.clone(), "M".to_string(), 0));

    for sw_type in ["L", "R"] {
        // sw_start and sw_end are both index of parent
        let mut sw_start;
        let mut sw_end;

        if sw_type == "R" {
            sw_start = parent.index(w0.max()) + 1;
            sw_end = sw_start + size - 1;
        } else {
            sw_end = parent.index(w0.min()) - 1;
            sw_start = sw_end - size + 1;
        }

        // distance is from 1 to max
        for sw_distance in 1..=max {
            if sw_start < 1 {
                break;
            }
            if sw_end > parent.size() {
                break;
            }

            let sw_intspan = parent.slice(sw_start, sw_end);

            if sw_intspan.size() < size {
                break;
            }

            windows.push((sw_intspan.clone(), sw_type.to_string(), sw_distance));

            if sw_type == "R" {
                sw_start = sw_end + 1;
                sw_end = sw_start + size - 1;
            } else {
                sw_end = sw_start - 1;
                sw_start = sw_end - size + 1;
            }
        }
    }

    windows
}

pub fn center_resize(parent: &IntSpan, intspan: &IntSpan, resize: i32) -> IntSpan {
    // find the middles of intspan
    let half_size = intspan.size() / 2;
    let mid_left = if half_size == 0 {
        intspan.at(1)
    } else {
        intspan.at(half_size)
    };
    let mid_right = if half_size == 0 {
        intspan.at(1)
    } else {
        intspan.at(half_size + 1)
    };
    let mid_left_idx = parent.index(mid_left);
    let mid_right_idx = parent.index(mid_right);

    // map to parent
    let half_resize = resize / 2;
    let mut left_idx = mid_left_idx - half_resize + 1;
    if left_idx < 1 {
        left_idx = 1;
    }
    let mut right_idx = mid_right_idx + half_resize - 1;
    if right_idx > parent.size() {
        right_idx = parent.size();
    }

    parent.slice(left_idx, right_idx)
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
    use crate::{center_resize, center_sw, thresholding_algo};
    use intspan::IntSpan;

    #[test]
    fn thresholding_sample() {
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

    #[test]
    fn test_center_resize() {
        // parent, runlist, resize, exp
        let tests = vec![
            ("1-500", "201", 100, "152-250"),
            ("1-500", "200", 100, "151-249"),
            ("1-500", "200-201", 100, "151-250"),
            ("1-500", "199-201", 100, "150-249"),
            ("1-500", "199-202", 100, "151-250"),
            ("1-500", "100-301", 100, "151-250"),
            ("1-500", "1", 100, "1-50"),
            ("1-500", "500", 100, "451-500"),
            ("1001-1500", "1200-1201", 100, "1151-1250"),
        ];

        for (parent, runlist, resize, exp) in tests {
            let intspan = IntSpan::from(runlist);
            let resized = center_resize(&IntSpan::from(parent), &intspan, resize);

            assert_eq!(resized.to_string(), exp);
        }
    }

    #[test]
    fn test_center_sw() {
        // parent, start, end, exp
        let tests = vec![
            ("1-9999", 500, 500, ("451-549", "M", 0, 3)),
            ("1-9999", 500, 800, ("600-699", "M", 0, 3)),
            ("1-9999", 101, 101, ("52-150", "M", 0, 2)),
        ];

        for (parent, start, end, exp) in tests {
            let windows = center_sw(&IntSpan::from(parent), start, end, 100, 1);

            assert_eq!(windows[0].0.to_string(), exp.0);
            assert_eq!(windows[0].1, exp.1);
            assert_eq!(windows[0].2, exp.2);
            assert_eq!(windows.len(), exp.3);
        }
    }
}
