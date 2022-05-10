use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("pos")
        .about("Add range files to positions")
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

    // processing each file
    for infile in args.values_of("infiles").unwrap() {
        let reader = reader(infile);

        for line in reader.lines().filter_map(|r| r.ok()) {
            let mut rg = Range::from_str(&line);
            if !rg.is_valid() {
                continue;
            }
            *rg.strand_mut() = "".to_string();

            let ctg_id = gars::find_one_z(&mut conn, &rg);
            if ctg_id.is_empty() {
                continue;
            }

            // Redis counter
            let counter = format!("cnt:pos:{}", ctg_id);
            let serial: isize = conn.incr(counter.clone(), 1).unwrap();
            let pos_id = format!("pos:{}:{}", ctg_id, serial);

            let _: () = redis::pipe()
                .hset(&pos_id, "chr_name", rg.chr())
                .ignore()
                .hset(&pos_id, "chr_start", *rg.start())
                .ignore()
                .hset(&pos_id, "chr_end", *rg.end())
                .ignore()
                .query(&mut conn)
                .unwrap();
        }

        // total number of pos
        let pos_count = gars::get_scan_count(&mut conn, "pos:*".to_string());
        println!("There are {} positions in total", pos_count);
    }

    Ok(())
}
