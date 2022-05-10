use clap::*;
use gars::*;
use intspan::*;
use std::io::BufRead;

use rust_lapper::{Interval, Lapper};

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("locate")
        .about("Locate the given ranges to the corresponding ctgs")
        .arg(
            Arg::new("ranges")
                .help("The given ranges, separating by spaces")
                .required(true)
                .min_values(1)
                .index(1),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .takes_value(false)
                .help("Treat ranges as filenames"),
        )
        .arg(
            Arg::new("rebuild")
                .long("rebuild")
                .short('r')
                .takes_value(false)
                .help("Rebuild the index of ctgs"),
        )
        .arg(
            Arg::new("zrange")
                .long("zrange")
                .short('z')
                .takes_value(false)
                .help("Use Redis ZRANGESTORE and ZINTER to locate the ctg"),
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
    let mut writer = intspan::writer(args.value_of("outfile").unwrap());

    // redis connection
    let mut conn = connect();

    // rebuild
    if args.is_present("rebuild") {
        gars::build_idx_ctg(&mut conn);
    }

    // all ranges
    let mut ranges: Vec<String> = vec![];
    if args.is_present("file") {
        for infile in args.values_of("ranges").unwrap() {
            let reader = reader(infile);
            for line in reader.lines().filter_map(|r| r.ok()) {
                let parts: Vec<&str> = line.split('\t').collect();
                ranges.push(parts.get(0).unwrap().to_string());
            }
        }
    } else {
        ranges = args
            .values_of("ranges")
            .unwrap()
            .into_iter()
            .map(|e| e.to_string())
            .collect();
    }

    // processing each file
    for range in ranges {
        let mut rg = Range::from_str(range.as_str());
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let ctg_id = if args.is_present("zrange") {
            gars::find_one_z(&mut conn, &rg)
        } else {
            gars::find_one_l(&mut conn, &rg)
        };

        if ctg_id.is_empty() {
            continue;
        }
        writer.write_fmt(format_args!("{}\t{}\t{}\n", range, rg.to_string(), ctg_id))?;
    }

    Ok(())
}
