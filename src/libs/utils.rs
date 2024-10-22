use lazy_static::lazy_static;
use regex::Regex;
use rust_lapper::Lapper;
use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

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

pub fn count_rg(
    lapper_of: &BTreeMap<String, Lapper<u32, String>>,
    ctg_id: &str,
    rg: &intspan::Range,
) -> i32 {
    if !lapper_of.contains_key(ctg_id) {
        eprintln!("{} not found in idx", ctg_id);
        return 0;
    }

    let lapper = lapper_of.get(ctg_id).unwrap();
    lapper.count(*rg.start() as u32, *rg.end() as u32) as i32
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

pub fn ctg_range_tuple(
    ranges_of: &BTreeMap<String, Vec<intspan::Range>>,
) -> Vec<(String, intspan::Range)> {
    let mut ctg_ranges: Vec<(String, intspan::Range)> = vec![];
    for k in ranges_of.keys() {
        for r in ranges_of.get(k).unwrap() {
            ctg_ranges.push((k.to_string(), r.clone()));
        }
    }
    ctg_ranges
}

/// Read peaks in the file
/// gc_content in this file aren't correct
pub fn read_peak(
    infile: &str,
    lapper_of: &BTreeMap<String, Lapper<u32, String>>,
) -> BTreeMap<String, Vec<(intspan::Range, String)>> {
    let reader = intspan::reader(infile);

    // ctg_id => [Range]
    let mut peaks_of: BTreeMap<String, Vec<(intspan::Range, String)>> = BTreeMap::new();

    // processing each line
    for line in reader.lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split('\t').collect();

        let mut rg = intspan::Range::from_str(parts[0]);
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let signal = parts[2];

        let ctg_id = crate::find_one_idx(lapper_of, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        peaks_of
            .entry(ctg_id)
            .and_modify(|v| v.push((rg, signal.to_string())))
            .or_default();
    }

    peaks_of
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

/// ```
/// assert_eq!(gams::round(4.364, 2), 4.36);
/// assert_eq!(gams::round(4.368, 2), 4.37);
/// ```
pub fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
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

    round(*cache.get(&field).unwrap(), 4)
}

pub fn gc_stat(gcs: &[f32]) -> (f32, f32, f32) {
    let mean = crate::mean(gcs);
    let stddev = crate::stddev(gcs);

    // coefficient of variation
    let cv = if mean == 0. || mean == 1. {
        0.
    } else if mean <= 0.5 {
        stddev / mean
    } else {
        stddev / (1. - mean)
    };

    // // Signal-to-noise ratio
    // let snr = if stddev == 0. {
    //     0.
    // } else if mean <= 0.5 {
    //     mean / stddev
    // } else {
    //     (1. - mean) / stddev
    // };

    (round(mean, 4), round(stddev, 4), round(cv, 4))
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
    let windows = crate::sliding(&intspan, size, step);

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

// Adopt from https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#create-a-parallel-pipeline
pub fn proc_ctg_p(
    ctgs: &Vec<crate::Ctg>,
    args: &clap::ArgMatches,
    proc_ctg: fn(&crate::Ctg, &clap::ArgMatches) -> String,
) -> crossbeam::channel::Receiver<String> {
    let parallel = *args.get_one::<usize>("parallel").unwrap();

    // Channel 1 - Contigs
    let (snd1, rcv1) = crossbeam::channel::bounded::<crate::Ctg>(10);
    // Channel 2 - Results
    let (snd2, rcv2) = crossbeam::channel::bounded::<String>(10);

    crossbeam::scope(|s| {
        //----------------------------
        // Reader thread
        //----------------------------
        s.spawn(|_| {
            for ctg in ctgs {
                snd1.send(ctg.clone()).unwrap();
            }
            // Close the channel - this is necessary to exit the for-loop in the worker
            drop(snd1);
        });

        //----------------------------
        // Worker threads
        //----------------------------
        for _ in 0..parallel {
            // Send to sink, receive from source
            let (sendr, recvr) = (snd2.clone(), rcv1.clone());
            // Spawn workers in separate threads
            s.spawn(move |_| {
                // Receive until channel closes
                for ctg in recvr.iter() {
                    let out_string = proc_ctg(&ctg, args);
                    sendr.send(out_string).unwrap();
                }
            });
        }
        // Close the channel, otherwise sink will never exit the for-loop
        drop(snd2);

        // //----------------------------
        // // Writer (main) thread
        // //----------------------------
        // for out_string in rcv2.iter() {
        //     writer.write_all(out_string.as_ref()).unwrap();
        // }
    })
    .unwrap();

    rcv2
}

// /// Don't use this
// pub fn find_one_l(conn: &mut redis::Connection, rg: &intspan::Range) -> String {
//     let lapper_bytes: Vec<u8> = conn.get(format!("idx:ctg:{}", rg.chr())).unwrap();
//
//     if lapper_bytes.is_empty() {
//         return "".to_string();
//     }
//
//     let lapper: Lapper<u32, String> = bincode::deserialize(&lapper_bytes).unwrap();
//     let res = lapper.find(*rg.start() as u32, *rg.end() as u32).next();
//
//     match res {
//         Some(iv) => iv.val.clone(),
//         None => "".to_string(),
//     }
// }
//
// /// Don't use this
// pub fn get_rg_seq(conn: &mut redis::Connection, rg: &intspan::Range) -> String {
//     let ctg_id = find_one_l(conn, rg);
//
//     if ctg_id.is_empty() {
//         return "".to_string();
//     }
//
//     let ctg = get_ctg(conn, &ctg_id);
//     let chr_start = ctg.chr_start;
//
//     let ctg_start = (rg.start() - chr_start + 1) as usize;
//     let ctg_end = (rg.end() - chr_start + 1) as usize;
//
//     let ctg_seq = get_seq(conn, &ctg_id);
//
//     // from <= x < to, zero-based
//     let seq = ctg_seq.get((ctg_start - 1)..(ctg_end)).unwrap();
//
//     String::from(seq)
// }
//
// /// This is an expensive operation
// pub fn get_gc_content(conn: &mut redis::Connection, rg: &intspan::Range) -> f32 {
//     let bucket = format!("cache:{}:{}", rg.chr(), rg.start() / 1000);
//     let field = rg.to_string();
//     let expire = 180;
//     let res = conn.hget(&bucket, &field).unwrap();
//
//     return match res {
//         Some(res) => {
//             let _: () = conn.expire(&bucket, expire).unwrap();
//
//             res
//         }
//         None => {
//             let seq = get_rg_seq(conn, rg);
//
//             let gc_content = if seq.is_empty() {
//                 0.
//             } else {
//                 bio::seq_analysis::gc::gc_content(seq.bytes())
//             };
//             let _: () = conn.hset(&bucket, &field, gc_content).unwrap();
//             let _: () = conn.expire(&bucket, expire).unwrap();
//
//             gc_content
//         }
//     };
// }
