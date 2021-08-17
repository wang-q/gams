use bio::io::fasta;
use clap::*;
use intspan::*;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sliding")
        .about("Sliding windows on a chromosome")
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

    let mut writer = writer(args.value_of("outfile").unwrap());

    writer.write_fmt(format_args!(
        "{}\t{}\n",
        "#range",
        "gc_content",
    ))?;

    // load the first seq
    let infile = args.value_of("infile").unwrap();
    let reader = reader(infile);
    let fa_in = fasta::Reader::new(reader);
    let record = fa_in.records().next().unwrap().unwrap();

    // intspan
    let seq = record.seq();
    let length = seq.len();
    let intspan = IntSpan::from_pair(1, length as i32);

    let windows = sliding(&intspan, size, step);

    for window in windows {
        let from = window.min() as usize;
        let to = window.max() as usize;

        // from <= x < to, zero-based
        let subseq = seq.get((from - 1)..(to)).unwrap();
        let gc_content = bio::seq_analysis::gc::gc_content(subseq);

        writer.write_fmt(format_args!(
            "{}:{}\t{}\n",
            record.id(),
            window.runlist(),
            gc_content
        ))?;
    }

    Ok(())
}

fn sliding(intspan: &IntSpan, size: i32, step: i32) -> Vec<IntSpan> {
    let mut windows = vec![];

    let mut start = 1;
    loop {
        let end = start + size - 1;
        if end > intspan.size() {
            break;
        }
        let window = slice(&intspan, start, end);
        start += step;

        windows.push(window);
    }

    windows
}

// TODO: switch to intspan.slice()
fn slice(intspan: &IntSpan, from: i32, to: i32) -> IntSpan {
    let lower = intspan.at(from);
    let upper = intspan.at(to);

    let new = IntSpan::from_pair(lower, upper);
    new.intersect(&intspan)
}
