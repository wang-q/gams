use clap::*;
use gars::*;
use intspan::*;
use std::collections::BTreeMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("locate")
        .about("Locate the given ranges to the corresponding ctgs")
        .after_help(
            r#"
It can also be used as a benchmark program.

"#,
        )
        .arg(
            Arg::new("ranges")
                .index(1)
                .num_args(1..)
                .help("The given ranges, separating by spaces"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("Treat ranges as filenames"),
        )
        .arg(
            Arg::new("rebuild")
                .long("rebuild")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Rebuild the index of ctgs"),
        )
        .arg(
            Arg::new("lapper")
                .long("lapper")
                .action(ArgAction::SetTrue)
                .help("Deserialize the index on request"),
        )
        .arg(
            Arg::new("zrange")
                .long("zrange")
                .action(ArgAction::SetTrue)
                .help("Use Redis ZRANGESTORE and ZINTER to locate the ctg"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    // redis connection
    let mut conn = connect();

    // rebuild
    if args.get_flag("rebuild") {
        gars::build_idx_ctg(&mut conn);
    }

    // all ranges
    let mut ranges: Vec<String> = vec![];
    if args.get_flag("file") {
        for infile in args.get_many::<String>("ranges").unwrap() {
            let reader = reader(infile);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split('\t').collect();
                ranges.push(parts.first().unwrap().to_string());
            }
        }
    } else {
        ranges = args
            .get_many::<String>("ranges")
            .unwrap()
            .map(|e| e.to_string())
            .collect();
    }

    // index of ctgs
    let mut lapper_of = BTreeMap::new();
    if !args.get_flag("lapper") || !args.get_flag("zrange") {
        lapper_of = gars::get_idx_ctg(&mut conn);
    }

    // processing each range
    for range in ranges {
        let mut rg = Range::from_str(range.as_str());
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let ctg_id = if args.get_flag("lapper") {
            gars::find_one_l(&mut conn, &rg)
        } else if args.get_flag("zrange") {
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
