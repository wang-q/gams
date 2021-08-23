use bio::io::fasta;
use clap::*;
use intspan::*;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sliding")
        .about("Sliding windows on a chromosome")
        .after_help(
            "\
             --step and --lag should be adjust simultaneously \
             ",
        )
        .arg(
            Arg::with_name("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
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

    let mut writer = writer(args.value_of("outfile").unwrap());

    writer.write_fmt(format_args!("{}\t{}\t{}\n", "#range", "gc_content", "signal"))?;

    // load the first seq
    let infile = args.value_of("infile").unwrap();
    let reader = reader(infile);
    let fa_in = fasta::Reader::new(reader);
    let record = fa_in.records().next().unwrap().unwrap();

    // intspan
    let seq = record.seq();
    let length = seq.len();
    let intspan = IntSpan::from_pair(1, length as i32);

    let windows = garr::sliding(&intspan, size, step);

    let mut gcs: Vec<f32> = Vec::with_capacity(windows.len());
    for window in &windows {
        let from = window.min() as usize;
        let to = window.max() as usize;

        // from <= x < to, zero-based
        let subseq = seq.get((from - 1)..(to)).unwrap();
        let gc_content = bio::seq_analysis::gc::gc_content(subseq);

        gcs.push(gc_content);
    }

    let signals = garr::thresholding_algo(&gcs, lag, threshold, influence);

    // outputs
    for i in 0..windows.len() {
        writer.write_fmt(format_args!(
            "{}:{}\t{}\t{}\n",
            record.id(),
            windows[i].runlist(),
            gcs[i],
            signals[i],
        ))?;
    }

    Ok(())
}
