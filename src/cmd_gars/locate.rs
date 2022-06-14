use clap::*;
use gars::*;
use intspan::*;
use std::collections::BTreeMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("locate")
        .about("Locate the given ranges to the corresponding ctgs")
        .after_help(
            r#"
It can also be used as a benchmark program.

"#,
        )
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
            Arg::new("lapper")
                .long("lapper")
                .takes_value(false)
                .help("Deserialize the index on request"),
        )
        .arg(
            Arg::new("zrange")
                .long("zrange")
                .takes_value(false)
                .help("Use Redis ZRANGESTORE and ZINTER to locate the ctg"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    // redis connection
    let mut conn = connect();

    // rebuild
    if args.contains_id("rebuild") {
        gars::build_idx_ctg(&mut conn);
    }

    // all ranges
    let mut ranges: Vec<String> = vec![];
    if args.contains_id("file") {
        for infile in args.get_many::<String>("ranges").unwrap() {
            let reader = reader(infile);
            for line in reader.lines().filter_map(|r| r.ok()) {
                let parts: Vec<&str> = line.split('\t').collect();
                ranges.push(parts.get(0).unwrap().to_string());
            }
        }
    } else {
        ranges = args
            .get_many::<String>("ranges")
            .unwrap()
            .into_iter()
            .map(|e| e.to_string())
            .collect();
    }

    // index of ctgs
    let mut lapper_of = BTreeMap::new();
    if !args.contains_id("lapper") || !args.contains_id("zrange") {
        lapper_of = gars::get_idx_ctg(&mut conn);
    }

    // processing each range
    for range in ranges {
        let mut rg = Range::from_str(range.as_str());
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let ctg_id = if args.contains_id("lapper") {
            gars::find_one_l(&mut conn, &rg)
        } else if args.contains_id("zrange") {
            gars::find_one_z(&mut conn, &rg)
        } else {
            gars::find_one_idx(&lapper_of, &rg)
        };

        if ctg_id.is_empty() {
            continue;
        }
        writer.write_fmt(format_args!("{}\t{}\t{}\n", range, rg, ctg_id))?;
    }

    Ok(())
}
