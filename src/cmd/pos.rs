use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pos")
        .about("Add range files to positions")
        .arg(
            Arg::with_name("infiles")
                .help("Sets the input file to use")
                .required(true)
                .min_values(1)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // redis connection
    let mut conn = connect();

    // pos in each contig
    // TODO: change to redis serial
    // let mut pos_serial: HashMap<String, i32> = HashMap::new();

    // processing each file
    for infile in args.values_of("infiles").unwrap() {
        let reader = reader(infile);

        for line in reader.lines().filter_map(|r| r.ok()) {
            let range = Range::from_str(&line);
            if !range.is_valid() {
                continue;
            }

            let ctg_id = garr::find_one(&mut conn, range.chr(), *range.start(), *range.end());
            if ctg_id.is_empty() {
                continue;
            }

            // https://docs.rs/redis/0.21.0/redis/fn.transaction.html
            let counter = format!("cnt:pos:{}", ctg_id);
            let serial: isize = conn.incr(counter.clone(), 1).unwrap();
            let pos_id = format!("pos:{}:{}", ctg_id, serial);

            let _: () = redis::pipe()
                .atomic()
                .hset(&pos_id, "chr_name", range.chr())
                .ignore()
                .hset(&pos_id, "chr_start", *range.start())
                .ignore()
                .hset(&pos_id, "chr_end", *range.end())
                .ignore()
                .query(&mut conn)
                .unwrap();

            // // TODO: change to atomic pipe
            // let serial = pos_serial.entry(ctg_id.clone()).or_insert(0);
            // *serial += 1;
            //
            // let pos_id = format!("pos:{}:{}", ctg_id, serial);
            //
            // let _: () = conn.hset(&pos_id, "chr_name", range.chr()).unwrap();
            // let _: () = conn.hset(&pos_id, "chr_start", *range.start()).unwrap();
            // let _: () = conn.hset(&pos_id, "chr_end", *range.end()).unwrap();
        }

        // total number of pos
        let pos_count = garr::get_scan_count(&mut conn, "pos:*".to_string());
        println!("There are {} positions in total", pos_count);
    }

    Ok(())
}
