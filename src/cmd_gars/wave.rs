use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::collections::HashMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("wave")
        .about("Add peaks of GC-waves")
        .after_help(
            r#"
Left-/right- wave lengths may be negative

"#,
        )
        .arg(
            Arg::new("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // opts
    let infile = args.value_of("infile").unwrap();

    // redis connection
    let mut conn = connect();

    // index of ctgs
    let lapper_of = gars::get_idx_ctg(&mut conn);

    // processing each line
    eprintln!("Loading peaks...");
    let reader = reader(infile);
    for line in reader.lines().filter_map(|r| r.ok()) {
        let parts: Vec<&str> = line.split('\t').collect();

        let mut rg = Range::from_str(parts[0]);
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let signal = parts[2];

        let ctg_id = gars::find_one_idx(&lapper_of, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        // Redis counter
        let counter = format!("cnt:peak:{}", ctg_id);
        let serial: isize = conn.incr(counter.clone(), 1).unwrap();
        let peak_id = format!("peak:{}:{}", ctg_id, serial);

        let length = rg.end() - rg.start() + 1;

        let _: () = redis::pipe()
            .hset(&peak_id, "chr_name", rg.chr())
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
    let n_peak = gars::get_scan_count(&mut conn, "peak:*".to_string());
    eprintln!("There are {} peaks", n_peak);

    // each ctg
    let ctgs: Vec<String> = gars::get_scan_vec(&mut conn, "ctg:*".to_string());
    eprintln!("Updating GC-content of peaks...");
    for ctg_id in &ctgs {
        let (chr_name, chr_start, chr_end) = gars::get_key_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        // local caches of GC-content for each ctg
        let mut cache: HashMap<String, f32> = HashMap::new();

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let seq: String = get_ctg_seq(&mut conn, ctg_id);

        let pattern = format!("peak:{}:*", ctg_id);
        let peaks: Vec<String> = gars::get_scan_vec(&mut conn, pattern);

        for peak_id in peaks {
            let (_, peak_start, peak_end) = gars::get_key_pos(&mut conn, &peak_id);

            let gc_content = cache_gc_content(
                &Range::from(&chr_name, peak_start, peak_end),
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
        let (chr_name, chr_start, chr_end) = gars::get_key_pos(&mut conn, &ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        let hash = gars::get_scan_int(
            &mut conn,
            format!("peak:{}:*", ctg_id),
            "chr_start".to_string(),
        );
        let mut peaks = hash.keys().cloned().collect::<Vec<String>>();
        eprintln!("\tThere are {} peaks", peaks.len());
        peaks.sort_by_key(|k| hash.get(k).unwrap());
        // eprintln!("{}\t{:#?}", ctg_id, peaks);

        if peaks.is_empty() {
            continue;
        }

        // left
        let (mut prev_signal, mut prev_gc): (String, f32) = conn
            .hget(peaks.first().unwrap(), &["signal", "gc"])
            .unwrap();
        let mut prev_end: i32 = chr_start;
        for i in 0..peaks.len() {
            let (cur_signal, cur_start, cur_end, cur_gc): (String, i32, i32, f32) = conn
                .hget(&peaks[i], &["signal", "chr_start", "chr_end", "gc"])
                .unwrap();

            // hset_multiple cannot be used because of the different value types
            let _: () = conn.hset(&peaks[i], "left_signal", prev_signal).unwrap();
            prev_signal = cur_signal.clone();

            let left_wave_length = cur_start - prev_end + 1;
            let _: () = conn
                .hset(&peaks[i], "left_wave_length", left_wave_length)
                .unwrap();
            prev_end = cur_end;

            let left_amplitude: f32 = (cur_gc - prev_gc).abs();
            let _: () = conn
                .hset(&peaks[i], "left_amplitude", left_amplitude)
                .unwrap();
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

            let _: () = conn.hset(&peaks[i], "right_signal", next_signal).unwrap();
            next_signal = cur_signal.clone();

            let right_wave_length = next_start - cur_end + 1;
            let _: () = conn
                .hset(&peaks[i], "right_wave_length", right_wave_length)
                .unwrap();
            next_start = cur_start;

            let right_amplitude: f32 = (cur_gc - next_gc).abs();
            let _: () = conn
                .hset(&peaks[i], "right_amplitude", right_amplitude)
                .unwrap();
            next_gc = cur_gc;
        }
    }

    Ok(())
}
