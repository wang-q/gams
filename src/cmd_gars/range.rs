use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
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
                .help("Sets the input file to use")
                .required(true)
                .min_values(1)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // redis connection
    let mut conn = connect();

    // index of ctgs
    let lapper_of = gars::get_idx_ctg(&mut conn);

    // processing each file
    for infile in args.values_of("infiles").unwrap() {
        let reader = reader(infile);

        for line in reader.lines().filter_map(|r| r.ok()) {
            let mut rg = Range::from_str(&line);
            if !rg.is_valid() {
                continue;
            }
            *rg.strand_mut() = "".to_string();

            let ctg_id = gars::find_one_idx(&lapper_of, &rg);
            if ctg_id.is_empty() {
                continue;
            }

            // Redis counter
            let serial: isize = conn.incr(format!("cnt:range:{}", ctg_id), 1).unwrap();
            let range_id = format!("range:{}:{}", ctg_id, serial);

            let _: () = redis::pipe()
                .hset(&range_id, "chr_name", rg.chr())
                .ignore()
                .hset(&range_id, "chr_start", *rg.start())
                .ignore()
                .hset(&range_id, "chr_end", *rg.end())
                .ignore()
                .query(&mut conn)
                .unwrap();
        }

        // total number of ranges
        let n_range = gars::get_scan_count(&mut conn, "range:*".to_string());
        println!("There are {} ranges in total", n_range);
    }

    Ok(())
}
