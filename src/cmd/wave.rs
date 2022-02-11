use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> App<'a> {
    App::new("wave")
        .about("Add peaks of GC-waves")
        .after_help(
            "\
    left-/right- wave length may be negative \
             ",
        )
        .arg(
            Arg::new("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let infile = args.value_of("infile").unwrap();

    // redis connection
    let mut conn = connect();

    // processing each line
    let reader = reader(infile);
    for line in reader.lines().filter_map(|r| r.ok()) {
        let parts: Vec<&str> = line.split('\t').collect();

        let mut rg = Range::from_str(parts[0]);
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let signal = parts[2];

        let ctg_id = garr::find_one(&mut conn, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        // Redis counter
        let counter = format!("cnt:peak:{}", ctg_id);
        let serial: isize = conn.incr(counter.clone(), 1).unwrap();
        let peak_id = format!("peak:{}:{}", ctg_id, serial);

        let length = rg.end() - rg.start() + 1;
        let gc_content = garr::get_gc_content(&mut conn, &rg);

        let _: () = redis::pipe()
            .hset(&peak_id, "chr_name", rg.chr())
            .ignore()
            .hset(&peak_id, "chr_start", *rg.start())
            .ignore()
            .hset(&peak_id, "chr_end", *rg.end())
            .ignore()
            .hset(&peak_id, "length", length)
            .ignore()
            .hset(&peak_id, "gc", gc_content)
            .ignore()
            .hset(&peak_id, "signal", signal)
            .ignore()
            .query(&mut conn)
            .unwrap();
    }

    // number of peaks
    let n_peak = garr::get_scan_count(&mut conn, "peak:*".to_string());
    println!("There are {} peaks", n_peak);

    // each ctg
    let ctgs: Vec<String> = garr::get_scan_vec(&mut conn, "ctg:*".to_string());
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in ctgs {
        let (chr_name, chr_start, chr_end) = garr::get_key_pos(&mut conn, &ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        let hash = garr::get_scan_int(
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
        let mut prev_signal: String = conn.hget(peaks.first().unwrap(), "signal").unwrap();
        let mut prev_end: i32 = conn.hget(&ctg_id, "chr_start").unwrap();
        let mut prev_gc: f32 = conn.hget(peaks.first().unwrap(), "gc").unwrap();
        for i in 0..peaks.len() {
            let cur_signal: String = conn.hget(&peaks[i], "signal").unwrap();
            let _: () = conn.hset(&peaks[i], "left_signal", prev_signal).unwrap();
            prev_signal = cur_signal.clone();

            let cur_start: i32 = conn.hget(&peaks[i], "chr_start").unwrap();
            let cur_end: i32 = conn.hget(&peaks[i], "chr_end").unwrap();
            let left_wave_length = cur_start - prev_end + 1;
            let _: () = conn
                .hset(&peaks[i], "left_wave_length", left_wave_length)
                .unwrap();
            prev_end = cur_end;

            let cur_gc: f32 = conn.hget(&peaks[i], "gc").unwrap();
            let left_amplitude: f32 = (cur_gc - prev_gc).abs();
            let _: () = conn
                .hset(&peaks[i], "left_amplitude", left_amplitude)
                .unwrap();
            prev_gc = cur_gc;
        }

        // right
        let mut next_signal: String = conn.hget(peaks.last().unwrap(), "signal").unwrap();
        let mut next_start: i32 = conn.hget(&ctg_id, "chr_end").unwrap();
        let mut next_gc: f32 = conn.hget(peaks.last().unwrap(), "gc").unwrap();
        for i in (0..peaks.len()).rev() {
            let cur_signal: String = conn.hget(&peaks[i], "signal").unwrap();
            let _: () = conn.hset(&peaks[i], "right_signal", next_signal).unwrap();
            next_signal = cur_signal.clone();

            let cur_start: i32 = conn.hget(&peaks[i], "chr_start").unwrap();
            let cur_end: i32 = conn.hget(&peaks[i], "chr_end").unwrap();
            let right_wave_length = next_start - cur_end + 1;
            let _: () = conn
                .hset(&peaks[i], "right_wave_length", right_wave_length)
                .unwrap();
            next_start = cur_start;

            let cur_gc: f32 = conn.hget(&peaks[i], "gc").unwrap();
            let right_amplitude: f32 = (cur_gc - next_gc).abs();
            let _: () = conn
                .hset(&peaks[i], "right_amplitude", right_amplitude)
                .unwrap();
            next_gc = cur_gc;
        }
    }

    Ok(())
}
