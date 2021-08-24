use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sliding")
        .about("Sliding windows along a chromosome")
        .after_help(
            "\
    --step and --lag should be adjust simultaneously. \
             ",
        )
        .arg(
            Arg::with_name("ctg")
                .long("ctg")
                .takes_value(true)
                .default_value("ctg:*")
                .empty_values(false)
                .help("Sets full name or prefix of contigs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::with_name("size")
                .long("size")
                .takes_value(true)
                .default_value("100")
                .empty_values(false),
        )
        .arg(
            Arg::with_name("step")
                .long("step")
                .takes_value(true)
                .default_value("50")
                .empty_values(false),
        )
        .arg(
            Arg::with_name("lag")
                .long("lag")
                .takes_value(true)
                .default_value("1000")
                .empty_values(false)
                .help("The lag of the moving window"),
        )
        .arg(
            Arg::with_name("threshold")
                .long("threshold")
                .takes_value(true)
                .default_value("3")
                .empty_values(false)
                .help("The z-score at which the algorithm signals"),
        )
        .arg(
            Arg::with_name("influence")
                .long("influence")
                .takes_value(true)
                .default_value("1")
                .empty_values(false)
                .help("The influence (between 0 and 1) of new signals on the mean and standard deviation"),
        )
        .arg(
            Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .empty_values(false)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let size: i32 = value_t!(args.value_of("size"), i32).unwrap_or_else(|e| {
        eprintln!("Need a integer for --size\n{}", e);
        std::process::exit(1)
    });
    let step: i32 = value_t!(args.value_of("step"), i32).unwrap_or_else(|e| {
        eprintln!("Need a integer for --step\n{}", e);
        std::process::exit(1)
    });
    let lag: usize = value_t!(args.value_of("lag"), usize).unwrap_or_else(|e| {
        eprintln!("Need a integer for --lag\n{}", e);
        std::process::exit(1)
    });
    let threshold: f32 = value_t!(args.value_of("threshold"), f32).unwrap_or_else(|e| {
        eprintln!("Need a float for --threshold\n{}", e);
        std::process::exit(1)
    });
    let influence: f32 = value_t!(args.value_of("influence"), f32).unwrap_or_else(|e| {
        eprintln!("Need a float for --influence\n{}", e);
        std::process::exit(1)
    });

    // redis connection
    let mut conn = connect();

    // headers
    let mut writer = writer(args.value_of("outfile").unwrap());
    writer.write_fmt(format_args!(
        "{}\t{}\t{}\n",
        "#range", "gc_content", "signal"
    ))?;

    // process each contig
    let ctgs: Vec<String> =
        garr::get_scan_vec(&mut conn, args.value_of("ctg").unwrap().to_string());
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in ctgs {
        let (chr_name, chr_start, chr_end) = garr::get_key_pos(&mut conn, &ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        let intspan = IntSpan::from_pair(chr_start, chr_end);
        let windows = garr::sliding(&intspan, size, step);

        let seq: String = conn.get(format!("seq:{}", ctg_id)).unwrap();

        let mut gcs: Vec<f32> = Vec::with_capacity(windows.len());
        for window in &windows {
            // converted to ctg index
            let from = intspan.index(window.min()) as usize;
            let to = intspan.index(window.max()) as usize;

            // from <= x < to, zero-based
            let subseq = seq.get((from - 1)..(to)).unwrap().bytes();
            let gc_content = bio::seq_analysis::gc::gc_content(subseq);
            gcs.push(gc_content);
        }

        let signals = garr::thresholding_algo(&gcs, lag, threshold, influence);

        // outputs
        for i in 0..windows.len() {
            writer.write_fmt(format_args!(
                "{}:{}\t{}\t{}\n",
                chr_name,
                windows[i].runlist(),
                gcs[i],
                signals[i],
            ))?;
        }
    }

    Ok(())
}
