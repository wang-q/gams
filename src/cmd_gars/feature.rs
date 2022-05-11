use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("feature")
        .about("Add genomic features from range files")
        .after_help(
            r#"
Serial - format!("cnt:feature:{}", ctg_id)
ID - format!("feature:{}:{}", ctg_id, serial)

"#,
        )
        .arg(
            Arg::new("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .short('t')
                .takes_value(true)
                .default_value("feature")
                .forbid_empty_values(true)
                .help("Feature tags"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // opts
    let infile = args.value_of("infile").unwrap();
    let tag = args.value_of("tag").unwrap();

    // redis connection
    let mut conn = connect();

    // index of ctgs
    let lapper_of = gars::get_idx_ctg(&mut conn);

    // processing each line
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
        let serial: isize = conn.incr(format!("cnt:feature:{}", ctg_id), 1).unwrap();
        let range_id = format!("feature:{}:{}", ctg_id, serial);

        let length = rg.end() - rg.start() + 1;

        let _: () = redis::pipe()
            .hset(&range_id, "chr_name", rg.chr())
            .ignore()
            .hset(&range_id, "chr_start", *rg.start())
            .ignore()
            .hset(&range_id, "chr_end", *rg.end())
            .ignore()
            .hset(&range_id, "length", length)
            .ignore()
            .hset(&range_id, "tag", tag)
            .ignore()
            .query(&mut conn)
            .unwrap();
    }

    // number of ranges
    let n_feature = gars::get_scan_count(&mut conn, "feature:*".to_string());
    println!("There are {} features", n_feature);

    Ok(())
}
