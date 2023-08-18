use clap::*;
use intspan::*;
use redis::Commands;
use std::collections::BTreeMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("range")
        .about("Add range files for counting")
        .after_help(
            r#"
Serial - format!("cnt:range:{}", ctg_id)
ID - format!("range:{}:{}", ctg_id, serial)

"#,
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
    let opt_size = *args.get_one::<usize>("size").unwrap();

    // redis connection
    let mut conn = gars::connect();

    // index of ctgs
    let lapper_of = gars::get_idx_ctg(&mut conn);

    // processing each file
    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = reader(infile);

        // ctg_id, Range
        // act as a sorter
        let mut ranges_of: BTreeMap<String, Vec<Range>> = BTreeMap::new();

        for line in reader.lines().map_while(Result::ok) {
            let mut rg = Range::from_str(&line);
            if !rg.is_valid() {
                continue;
            }
            *rg.strand_mut() = "".to_string();

            let ctg_id = gars::find_one_idx(&lapper_of, &rg);
            if ctg_id.is_empty() {
                continue;
            }

            ranges_of
                .entry(ctg_id)
                .and_modify(|v| v.push(rg))
                .or_insert(Vec::new());
        }

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
        // For each ctg, we increase and set the counter only once
        let mut serial_of: BTreeMap<String, i32> = BTreeMap::new();

        let mut batch = &mut redis::pipe();

        for (i, (ctg_id, rg)) in ctg_ranges.iter().enumerate() {
            // prompts
            if i > 1 && i % opt_size == 0 {
                let _: () = batch.query(&mut conn).unwrap();
                batch.clear();
            }
            if i > 1 && i % (opt_size * 10) == 0 {
                eprintln!("Read {} records", i);
            }

            // serial and id
            if !serial_of.contains_key(ctg_id) {
                // Redis counter
                let serial: i32 = conn.incr(format!("cnt:range:{}", ctg_id), 0).unwrap();
                serial_of.insert(ctg_id.to_string(), serial);
            }

            let serial = serial_of.get_mut(ctg_id).unwrap();
            *serial += 1;
            let range_id = format!("range:{}:{}", ctg_id, serial);

            batch = batch
                .hset(&range_id, "chr_id", rg.chr())
                .ignore()
                .hset(&range_id, "chr_start", *rg.start())
                .ignore()
                .hset(&range_id, "chr_end", *rg.end())
                .ignore();
        }

        // There could left records in stmts
        {
            let _: () = batch.query(&mut conn).unwrap();
            batch.clear();
        }

        // Write increased serials back to Redis
        for (ctg_id, serial) in &serial_of {
            let _: () = conn.set(format!("cnt:range:{}", ctg_id), serial).unwrap();
        }
    }

    Ok(())
}
