use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> App<'a> {
    App::new("range")
        .about("Add ranges")
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
                .default_value("range")
                .forbid_empty_values(true)
                .help("Range tags"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let infile = args.value_of("infile").unwrap();
    let tag = args.value_of("tag").unwrap();

    // redis connection
    let mut conn = connect();

    // processing each line
    let reader = reader(infile);
    for line in reader.lines().filter_map(|r| r.ok()) {
        let mut rg = Range::from_str(&line);
        if !rg.is_valid() {
            continue;
        }
        *rg.strand_mut() = "".to_string();

        let ctg_id = garr::find_one(&mut conn, &rg);
        if ctg_id.is_empty() {
            continue;
        }

        // Redis counter
        let counter = format!("cnt:range:{}", ctg_id);
        let serial: isize = conn.incr(counter.clone(), 1).unwrap();
        let range_id = format!("range:{}:{}", ctg_id, serial);

        let length = rg.end() - rg.start() + 1;
        let gc_content = garr::get_gc_content(&mut conn, &rg);

        let _: () = redis::pipe()
            .hset(&range_id, "chr_name", rg.chr())
            .ignore()
            .hset(&range_id, "chr_start", *rg.start())
            .ignore()
            .hset(&range_id, "chr_end", *rg.end())
            .ignore()
            .hset(&range_id, "length", length)
            .ignore()
            .hset(&range_id, "gc", gc_content)
            .ignore()
            .hset(&range_id, "tag", tag)
            .ignore()
            .query(&mut conn)
            .unwrap();
    }

    // number of ranges
    let n_range = garr::get_scan_count(&mut conn, "range:*".to_string());
    println!("There are {} ranges", n_range);

    Ok(())
}
