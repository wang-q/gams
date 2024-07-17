use clap::*;
use gams::*;
use intspan::*;
use redis::Commands;
use std::collections::HashMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("peak")
        .about("Add peaks of GC-waves")
        .after_help(
            r#"
Serial - format!("cnt:peak:{}", ctg_id)
ID - format!("peak:{}:{}", ctg_id, serial)

Left-/right- wave lengths may be negative

"#,
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
    let mut conn = connect();

    // index of ctgs
    let lapper_of = gams::get_idx_ctg(&mut conn);

    // processing each line
    eprintln!("Loading peaks...");
    let reader = reader(infile);
    for line in reader.lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split('\t').collect();

        let mut rg = Range::from_str(parts[0]);
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let signal = parts[2];

        let ctg_id = gams::find_one_idx(&lapper_of, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        // Redis counter
        let serial: i32 = conn.incr(format!("cnt:peak:{}", ctg_id), 1).unwrap();
        let peak_id = format!("peak:{}:{}", ctg_id, serial);

        let length = rg.end() - rg.start() + 1;

        // hset_multiple cannot be used because of the different value types
        let _: () = redis::pipe()
            .hset(&peak_id, "chr_id", rg.chr())
            .ignore()
            .hset(&peak_id, "chr_start", *rg.start())
            .ignore()
            .hset(&peak_id, "chr_end", *rg.end())
            .ignore()
            .hset(&peak_id, "length", length)
            .ignore()
            .hset(&peak_id, "signal", signal)
            .ignore()
            .query(&mut conn)
            .unwrap();
    }

    // number of peaks
    let n_peak = gams::get_scan_count(&mut conn, "peak:*");
    eprintln!("There are {} peaks\n", n_peak);

    // each ctg
    let ctgs: Vec<String> = gams::get_scan_vec(&mut conn, "ctg:*");
    eprintln!("Updating GC-content of peaks...");
    for ctg_id in &ctgs {
        let (chr_id, chr_start, chr_end) = gams::get_key_pos(&mut conn, ctg_id);

        // local caches of GC-content for each ctg
        let mut cache: HashMap<String, f32> = HashMap::new();

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let seq: String = get_seq(&mut conn, ctg_id);

        // scan_match() is an expensive op. Replace with cnt
        // let pattern = format!("peak:{}:*", ctg_id);
        // let peaks: Vec<String> = gams::get_scan_vec(&mut conn, pattern);
        let peaks: Vec<String> = get_vec_feature(&mut conn, ctg_id);
        for peak_id in peaks {
            let (_, peak_start, peak_end) = gams::get_key_pos(&mut conn, &peak_id);

            let gc_content = cache_gc_content(
                &Range::from(&chr_id, peak_start, peak_end),
                &parent,
                &seq,
                &mut cache,
            );
            let _: () = conn.hset(&peak_id, "gc", gc_content).unwrap();
        }
    }

    // each ctg
    eprintln!("Updating relationships of peaks...");
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in &ctgs {
        let (chr_id, chr_start, chr_end) = gams::get_key_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

        // All peaks in this ctg
        let mut peaks: Vec<String> = get_vec_feature(&mut conn, ctg_id);
        eprintln!("\tThere are {} peaks", peaks.len());

        if peaks.is_empty() {
            continue;
        }

        // sort peaks
        let mut chr_start_of: HashMap<String, i32> = HashMap::new();
        for key in &peaks {
            let val: i32 = conn.hget(key, "chr_start".to_string()).unwrap();
            chr_start_of.insert(key.clone(), val);
        }
        peaks.sort_by_key(|k| chr_start_of.get(k).unwrap());
        // eprintln!("{}\t{:#?}", ctg_id, peaks);

        // left
        let (mut prev_signal, mut prev_gc): (String, f32) = conn
            .hget(peaks.first().unwrap(), &["signal", "gc"])
            .unwrap();
        let mut prev_end: i32 = chr_start;
        for peak in &peaks {
            let (cur_signal, cur_start, cur_end, cur_gc): (String, i32, i32, f32) = conn
                .hget(peak, &["signal", "chr_start", "chr_end", "gc"])
                .unwrap();

            let left_wave_length = cur_start - prev_end + 1;
            let left_amplitude: f32 = (cur_gc - prev_gc).abs();

            // hset_multiple cannot be used because of the different value types
            let _: () = redis::pipe()
                .hset(peak, "left_signal", &prev_signal)
                .ignore()
                .hset(peak, "left_wave_length", left_wave_length)
                .ignore()
                .hset(peak, "left_amplitude", left_amplitude)
                .ignore()
                .query(&mut conn)
                .unwrap();

            prev_signal = cur_signal.clone();
            prev_end = cur_end;
            prev_gc = cur_gc;
        }

        // right
        let (mut next_signal, mut next_gc): (String, f32) =
            conn.hget(peaks.last().unwrap(), &["signal", "gc"]).unwrap();
        let mut next_start: i32 = chr_end;
        for i in (0..peaks.len()).rev() {
            let (cur_signal, cur_start, cur_end, cur_gc): (String, i32, i32, f32) = conn
                .hget(&peaks[i], &["signal", "chr_start", "chr_end", "gc"])
                .unwrap();

            let right_wave_length = next_start - cur_end + 1;
            let right_amplitude: f32 = (cur_gc - next_gc).abs();

            let _: () = redis::pipe()
                .hset(&peaks[i], "right_signal", &next_signal)
                .ignore()
                .hset(&peaks[i], "right_wave_length", right_wave_length)
                .ignore()
                .hset(&peaks[i], "right_amplitude", right_amplitude)
                .ignore()
                .query(&mut conn)
                .unwrap();

            next_signal = cur_signal.clone();
            next_start = cur_start;
            next_gc = cur_gc;
        }
    }

    Ok(())
}
