use clap::*;
use gars::*;
use intspan::*;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("sliding")
        .about("Sliding windows along a chromosome")
        .after_help(
            "\
    --step and --lag should be adjust simultaneously. \
             ",
        )
        .arg(
            Arg::new("ctg")
                .long("ctg")
                .takes_value(true)
                .default_value("ctg:*")
                .forbid_empty_values(true)
                .help("Sets full name or prefix of contigs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .takes_value(true)
                .default_value("100")
                .forbid_empty_values(true)
        )
        .arg(
            Arg::new("step")
                .long("step")
                .takes_value(true)
                .default_value("50")
                .forbid_empty_values(true)
        )
        .arg(
            Arg::new("lag")
                .long("lag")
                .takes_value(true)
                .default_value("1000")
                .forbid_empty_values(true)
                .help("The lag of the moving window"),
        )
        .arg(
            Arg::new("threshold")
                .long("threshold")
                .takes_value(true)
                .default_value("3")
                .forbid_empty_values(true)
                .help("The z-score at which the algorithm signals"),
        )
        .arg(
            Arg::new("influence")
                .long("influence")
                .takes_value(true)
                .default_value("1")
                .forbid_empty_values(true)
                .help("The influence (between 0 and 1) of new signals on the mean and standard deviation"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .forbid_empty_values(true)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // opts
    let size: i32 = args.value_of_t("size").unwrap_or_else(|e| {
        eprintln!("Need a integer for --size\n{}", e);
        std::process::exit(1)
    });
    let step: i32 = args.value_of_t("step").unwrap_or_else(|e| {
        eprintln!("Need a integer for --step\n{}", e);
        std::process::exit(1)
    });
    let lag: usize = args.value_of_t("lag").unwrap_or_else(|e| {
        eprintln!("Need a integer for --lag\n{}", e);
        std::process::exit(1)
    });
    let threshold: f32 = args.value_of_t("threshold").unwrap_or_else(|e| {
        eprintln!("Need a float for --threshold\n{}", e);
        std::process::exit(1)
    });
    let influence: f32 = args.value_of_t("influence").unwrap_or_else(|e| {
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
        gars::get_scan_vec(&mut conn, args.value_of("ctg").unwrap().to_string());
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in ctgs {
        let (chr_name, chr_start, chr_end) = gars::get_key_pos(&mut conn, &ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let windows = gars::sliding(&parent, size, step);

        let ctg_seq: String = get_ctg_seq(&mut conn, &ctg_id);

        let mut gcs: Vec<f32> = Vec::with_capacity(windows.len());
        for window in &windows {
            // converted to ctg index
            let from = parent.index(window.min()) as usize;
            let to = parent.index(window.max()) as usize;

            // from <= x < to, zero-based
            let subseq = ctg_seq.get((from - 1)..(to)).unwrap().bytes();
            let gc_content = bio::seq_analysis::gc::gc_content(subseq);
            gcs.push(gc_content);
        }

        let signals = gars::thresholding_algo(&gcs, lag, threshold, influence);

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
