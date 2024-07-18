use clap::*;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("feature")
        .about("Add genomic features from a range file")
        .after_help(
            r###"
feature:
    cnt:feature:{ctg_id}        => serial
    feature:{ctg_id}:{serial}   => Feature
    bin:feature:{ctg_id}        => Redis SET Feature

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
    let mut conn = gams::connect();

    // index of ctgs
    let lapper_of = gams::get_idx_ctg(&mut conn);

    {
        // ctg_id => [Range]
        // act as a sorter
        let ranges_of = gams::read_range(infile, &lapper_of);

        // (ctg_id, Range)
        let mut ctg_ranges: Vec<(String, intspan::Range)> = vec![];
        for k in ranges_of.keys() {
            for r in ranges_of.get(k).unwrap() {
                ctg_ranges.push((k.to_string(), r.clone()));
            }
        }

        // total number of ranges
        eprintln!("There are {} features in this file", ctg_ranges.len());

        // start serial of each ctg
        // To minimize expensive Redis operations, locally increment the serial number
        // For each ctg, we increase the counter only once
        let mut serial_of: BTreeMap<String, i32> = BTreeMap::new();

        let mut batch = &mut redis::pipe();

        for (i, (ctg_id, rg)) in ctg_ranges.iter().enumerate() {
            // prompts
            if i > 1 && i % opt_size == 0 {
                let _: () = batch.query(&mut conn).unwrap();
                batch.clear();
            }
            if i > 1 && i % (opt_size * 10) == 0 {
                eprintln!("Insert {} records", i);
            }

            // serial and id
            if !serial_of.contains_key(ctg_id) {
                let cnt = ranges_of.get(ctg_id).unwrap().len() as i32;
                // Redis counter
                // increased serial by cnt
                let serial =
                    gams::incr_serial_n(&mut conn, &format!("cnt:feature:{ctg_id}"), cnt) as i32;

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
                ctg_id: ctg_id.clone(),
                tag: opt_tag.to_string(),
            };

            // Add serialized struct Feature to a Redis set
            let set_name = format!("bin:feature:{ctg_id}");
            let feature_bytes = bincode::serialize(&feature).unwrap();

            batch = batch
                .set(&feature_id, feature_bytes.clone())
                .ignore()
                .sadd(&set_name, feature_bytes)
                .ignore();
        }

        // Possible remaining records in the batch
        {
            let _: () = batch.query(&mut conn).unwrap();
            batch.clear();
        }
    }

    // number of features
    let n_feature = gams::get_scan_count(&mut conn, "feature:*");
    eprintln!("There are {} features in the database", n_feature);

    Ok(())
}
