use clap::*;
use intspan::*;
use redis::Commands;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("feature")
        .about("Add genomic features from a range file")
        .after_help(
            r#"
Serial - format!("cnt:feature:{}", ctg_id)
ID - format!("feature:{}:{}", ctg_id, serial)

"#,
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
    let mut conn = gars::connect();

    // index of ctgs
    let lapper_of = gars::get_idx_ctg(&mut conn);

    {
        // ctg_id => [Range]
        // act as a sorter
        let ranges_of = gars::read_range(infile, &lapper_of);

        // ctg_id, Range
        let mut ctg_ranges: Vec<(String, Range)> = vec![];
        for k in ranges_of.keys() {
            for r in ranges_of.get(k).unwrap() {
                ctg_ranges.push((k.to_string(), r.clone()));
            }
        }

        // total number of ranges
        eprintln!("There are {} ranges in this file", ctg_ranges.len());

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
                // increased serial
                let serial: i32 = conn.incr(format!("cnt:feature:{}", ctg_id), cnt).unwrap();

                // here we start
                serial_of.insert(ctg_id.to_string(), serial - cnt);
            }

            let serial = serial_of.get_mut(ctg_id).unwrap();
            *serial += 1;
            let feature_id = format!("feature:{}:{}", ctg_id, serial);

            batch = batch
                .hset(&feature_id, "chr_id", rg.chr())
                .ignore()
                .hset(&feature_id, "chr_start", *rg.start())
                .ignore()
                .hset(&feature_id, "chr_end", *rg.end())
                .ignore()
                .hset(&feature_id, "length", rg.end() - rg.start() + 1)
                .ignore()
                .hset(&feature_id, "tag", opt_tag)
                .ignore();
        }

        // There could left records in the batch
        {
            let _: () = batch.query(&mut conn).unwrap();
            batch.clear();
        }
    }

    // // number of ranges
    // let n_feature = gars::get_scan_count(&mut conn, "feature:*".to_string());
    // eprintln!("There are {} features", n_feature);

    Ok(())
}
