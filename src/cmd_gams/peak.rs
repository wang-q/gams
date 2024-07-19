use clap::*;
use std::collections::{BTreeMap, HashMap};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("peak")
        .about("Add peaks of GC-waves")
        .after_help(
            r###"
Left-/right- wave lengths may be negative

"###,
        )
        .arg(
            Arg::new("infile")
                .index(1)
                .num_args(1)
                .help("Sets the input file to use"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let infile = args.get_one::<String>("infile").unwrap();

    // redis connection
    let mut conn = gams::connect();

    // index of ctgs
    let lapper_of = gams::get_idx_ctg(&mut conn);

    // ctg_id => [(Range, signal)]
    eprintln!("Loading peaks...");
    let peaks_of = gams::read_peak(infile, &lapper_of);

    // start serial of each ctg
    // To minimize expensive Redis operations, locally increment the serial number
    // For each ctg, we increase the counter in Redis only once
    let mut serial_of: BTreeMap<String, i32> = BTreeMap::new();
    let mut s_peaks_of: BTreeMap<String, Vec<gams::Peak>> = Default::default();
    for ctg_id in peaks_of.keys() {
        let (chr_id, chr_start, chr_end) = gams::get_ctg_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

        // tuple with 2 members
        let mut peaks_t2 = peaks_of.get(ctg_id).unwrap().clone();
        peaks_t2.sort_by_cached_key(|el| el.0.start);

        if peaks_t2.is_empty() {
            continue;
        }

        // number of peaks
        let n_peak = peaks_t2.len() as i32;
        eprintln!("There are {} peaks\n", n_peak);

        let parent = intspan::IntSpan::from_pair(chr_start, chr_end);
        let seq: String = gams::get_seq(&mut conn, ctg_id);

        // local caches of GC-content for each ctg
        let mut cache: HashMap<String, f32> = HashMap::new();

        // each peak
        let mut peaks: Vec<gams::Peak> = Default::default();
        for tp in &peaks_t2 {
            // serial and id
            if !serial_of.contains_key(ctg_id) {
                // Redis counter
                // increase serial by cnt
                let serial =
                    gams::incr_serial_n(&mut conn, &format!("cnt:peak:{ctg_id}"), n_peak) as i32;

                // here we start
                serial_of.insert(ctg_id.to_string(), serial - n_peak);
            }
            let serial = serial_of.get_mut(ctg_id).unwrap();
            *serial += 1;
            let peak_id = format!("peak:{ctg_id}:{serial}");

            let gc_content = gams::cache_gc_content(&tp.0, &parent, &seq, &mut cache);

            let peak = gams::Peak {
                id: peak_id.clone(),
                range: tp.0.to_string(),
                length: tp.0.end() - tp.0.start() + 1,
                signal: tp.1.clone(),
                gc: gc_content,
                left_signal: None,
                left_wave_length: None,
                left_amplitude: None,
                right_signal: None,
                right_wave_length: None,
                right_amplitude: None,
            };
            peaks.push(peak);
        }

        s_peaks_of.insert(ctg_id.clone(), peaks);
    }

    // each ctg
    eprintln!("Updating relationships of peaks...");
    eprintln!("{} contigs to be processed", s_peaks_of.len());
    for ctg_id in s_peaks_of.keys().cloned().collect::<Vec<_>>().iter() {
        let (chr_id, chr_start, chr_end) = gams::get_ctg_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

        // All peaks in this ctg, sorted
        let peaks = s_peaks_of.get_mut(ctg_id).unwrap();
        eprintln!("\tThere are {} peaks", peaks.len());

        // left
        let mut prev_signal = peaks.first().unwrap().signal.clone();
        let mut prev_gc = peaks.first().unwrap().gc;
        let mut prev_end: i32 = chr_start;
        for i in 0..peaks.len() {
            let peak = peaks.get_mut(i).unwrap();

            let rg = intspan::Range::from_str(&peak.range);
            let cur_signal = peak.signal.clone();
            let cur_start = rg.start;
            let cur_end = rg.end;
            let cur_gc = peak.gc;

            let left_wave_length = cur_start - prev_end + 1;
            let left_amplitude: f32 = (cur_gc - prev_gc).abs();

            peak.left_signal = Some(prev_signal.clone());
            peak.left_wave_length = Some(left_wave_length);
            peak.left_amplitude = Some(left_amplitude);

            prev_signal.clone_from(&cur_signal);
            prev_end = cur_end;
            prev_gc = cur_gc;
        }

        // right
        let mut next_signal = peaks.last().unwrap().signal.clone();
        let mut next_gc = peaks.last().unwrap().gc;
        let mut next_start: i32 = chr_end;
        for i in (0..peaks.len()).rev() {
            let peak = peaks.get_mut(i).unwrap();

            let rg = intspan::Range::from_str(&peak.range);
            let cur_signal = peak.signal.clone();
            let cur_start = rg.start;
            let cur_end = rg.end;
            let cur_gc = peak.gc;

            let right_wave_length = next_start - cur_end + 1;
            let right_amplitude: f32 = (cur_gc - next_gc).abs();

            peak.right_signal = Some(next_signal.clone());
            peak.right_wave_length = Some(right_wave_length);
            peak.right_amplitude = Some(right_amplitude);

            next_signal.clone_from(&cur_signal);
            next_start = cur_start;
            next_gc = cur_gc;
        }
    }

    let writer = intspan::writer("stdout");
    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(writer);
    for ctg_id in s_peaks_of.keys() {
        let peaks = s_peaks_of.get(ctg_id).unwrap();
        for peak in peaks {
            tsv_wtr.serialize(peak).unwrap();
        }
    }

    Ok(())
}
