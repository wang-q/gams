use clap::*;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("rg")
        .about("Add range files for counting")
        .after_help(
            r###"
"###,
        )
        .arg(
            Arg::new("infiles")
                .index(1)
                .num_args(1..)
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
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let opt_size = *args.get_one::<usize>("size").unwrap();

    // redis connection
    let mut conn = gams::connect();

    // index of ctgs
    let lapper_of = gams::get_idx_ctg(&mut conn);

    // processing each file
    for infile in args.get_many::<String>("infiles").unwrap() {
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
        eprintln!("There are {} rgs in this file", ctg_ranges.len());

        // start serial of each ctg
        // To minimize expensive Redis operations, locally increment the serial number
        // For each ctg, we increase the counter only once
        let mut serial_of: BTreeMap<String, i32> = BTreeMap::new();

        let mut batch = &mut redis::pipe();

        for (i, (ctg_id, range)) in ctg_ranges.iter().enumerate() {
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
                // increase serial by cnt
                let serial =
                    gams::incr_serial_n(&mut conn, &format!("cnt:rg:{ctg_id}"), cnt) as i32;

                // here we start
                serial_of.insert(ctg_id.to_string(), serial - cnt);
            }

            let serial = serial_of.get_mut(ctg_id).unwrap();
            *serial += 1;
            let rg_id = format!("rg:{ctg_id}:{serial}");

            let rg = gams::Rg {
                id: rg_id.clone(),
                range: range.to_string(),
            };
            let json = serde_json::to_string(&rg).unwrap();
            batch = batch.set(&rg_id, json.clone()).ignore();
        }
        // Possible remaining records in the batch
        {
            let _: () = batch.query(&mut conn).unwrap();
            batch.clear();
        }
    }

    // number of rgs
    let n_rg = gams::get_scan_count(&mut conn, "rg:*");
    eprintln!("There are {} rgs in the database", n_rg);

    Ok(())
}
