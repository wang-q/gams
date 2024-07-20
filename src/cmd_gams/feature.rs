use clap::*;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("feature")
        .about("Add genomic features from a range file")
        .after_help(
            r###"
Please process multiple files separately, as you will have to tag each file

"###,
        )
        .arg(
            Arg::new("infile")
                .index(1)
                .num_args(1)
                .help("Sets the input file to use"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .num_args(1)
                .default_value("100")
                .value_parser(value_parser!(usize))
                .help("Batch size for one Redis submission"),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .short('t')
                .num_args(1)
                .default_value("feature")
                .help("Feature tags"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let infile = args.get_one::<String>("infile").unwrap();
    let opt_tag = args.get_one::<String>("tag").unwrap().as_str();
    let opt_size = *args.get_one::<usize>("size").unwrap();

    // redis connection
    let mut conn = gams::Conn::with_size(opt_size);

    // ctg_id => [Range]
    // act as a sorter
    let ranges_of = {
        // index of ctgs
        let lapper_of = conn.get_idx_ctg();
        gams::read_range(infile, &lapper_of)
    };

    // (ctg_id, Range)
    let ctg_ranges = gams::ctg_range_tuple(&ranges_of);

    // total number of ranges
    eprintln!("There are {} features in this file", ctg_ranges.len());

    // start serial of each ctg
    // To minimize expensive Redis operations, locally increment the serial number
    // For each ctg, we increase the counter in Redis only once
    let mut serial_of: BTreeMap<String, i32> = BTreeMap::new();

    for (i, (ctg_id, rg)) in ctg_ranges.iter().enumerate() {
        // prompts
        if i > 1 && i % (opt_size * 10) == 0 {
            eprintln!("Insert {} records", i);
        }

        // serial and id
        if !serial_of.contains_key(ctg_id) {
            let cnt = ranges_of.get(ctg_id).unwrap().len() as i32;
            // Redis counter
            // increase serial by cnt
            let serial = conn.incr_sn_n(&format!("cnt:feature:{ctg_id}"), cnt);

            // here we start
            serial_of.insert(ctg_id.to_string(), serial - cnt);
        }

        let serial = serial_of.get_mut(ctg_id).unwrap();
        *serial += 1;
        let feature_id = format!("feature:{ctg_id}:{serial}");

        let feature = gams::Feature {
            id: feature_id.clone(),
            range: rg.to_string(),
            length: rg.end() - rg.start() + 1,
            tag: opt_tag.to_string(),
        };

        let json = serde_json::to_string(&feature).unwrap();
        conn.pipe_add(&feature_id, &json);
    }
    conn.pipe_submit(); // Possible remaining records in the pipe

    let n_feature = conn.get_scan_count("feature:*");
    eprintln!("There are {} features in the database", n_feature);

    Ok(())
}
