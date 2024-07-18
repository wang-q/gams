use std::collections::{BTreeMap, HashMap};
use rust_lapper::{Interval, Lapper};
use std::io::{BufRead, Read};
use lazy_static::lazy_static;
use regex::Regex;

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

        let ctg_id = crate::find_one_idx(lapper_of, &rg);
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
/// Read peaks in the file
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

        let ctg_id = crate::find_one_idx(&lapper_of, &rg);
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

    crate::gc_stat(&gcs)
}
