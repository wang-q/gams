use clap::*;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("locate")
        .about("Locate the given ranges to the corresponding ctgs")
        .after_help(
            r###"
* `--seq` might not be useful, just in case that you can't access the fasta files
* To use `--count`, `gams rg` should have inserted .rg files
* `--seq` will untick `--count`

"###,
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
                .help("Rebuild the index of ctgs and rgs"),
        )
        .arg(
            Arg::new("seq")
                .long("seq")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Extracts sequences defined by the range(s)"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Count overlaps with the ranges stored in gams"),
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

    let is_rebuild = args.get_flag("rebuild");
    let is_file = args.get_flag("file");
    let is_seq = args.get_flag("seq");
    let is_count = if is_seq {
        false
    } else {
        args.get_flag("count")
    };

    // redis connection
    let mut conn = gams::Conn::new();

    // rebuild
    if is_rebuild {
        conn.build_idx_ctg();
    }

    // all ranges
    let mut rgs: Vec<String> = vec![];
    if is_file {
        for infile in args.get_many::<String>("ranges").unwrap() {
            let reader = intspan::reader(infile);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split('\t').collect();
                rgs.push(parts.first().unwrap().to_string());
            }
        }
    } else {
        rgs = args
            .get_many::<String>("ranges")
            .unwrap()
            .map(|e| e.to_string())
            .collect();
    }

    // index of ctgs
    let lapper_ctg_of = conn.get_idx_ctg();
    let lapper_rg_of = if is_count {
        Some(conn.get_idx_rg())
    } else {
        None
    };

    // processing each range
    for rg in &rgs {
        let mut range = intspan::Range::from_str(rg);
        if !range.is_valid() {
            continue;
        }
        *range.strand_mut() = "".to_string();

        let ctg_id = gams::find_one_idx(&lapper_ctg_of, &range);

        if ctg_id.is_empty() {
            continue;
        }

        if is_seq {
            let ctg = conn.get_ctg(&ctg_id);
            let chr_start = ctg.chr_start;

            let ctg_start = (range.start() - chr_start + 1) as usize;
            let ctg_end = (range.end() - chr_start + 1) as usize;

            let ctg_seq = conn.get_seq(&ctg_id);
            // from <= x < to, zero-based
            let seq = ctg_seq.get((ctg_start - 1)..(ctg_end)).unwrap();
            writer.write_fmt(format_args!(">{}\n{}\n", rg, seq))?;
        } else if is_count{
            let cnt = gams::count_rg( &lapper_rg_of.unwrap(), &ctg_id, &range);
            writer.write_fmt(format_args!("{}\t{}\n", rg, cnt))?;
        } else {
            writer.write_fmt(format_args!("{}\t{}\n", rg, ctg_id))?;
        }
    }

    Ok(())
}
