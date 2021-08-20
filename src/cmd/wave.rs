use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("wave")
        .about("Add peaks of GC-waves")
        .arg(
            Arg::with_name("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let infile = args.value_of("infile").unwrap();

    // redis connection
    let mut conn = connect();

    // processing each line
    let reader = reader(infile);
    for line in reader.lines().filter_map(|r| r.ok()) {
        let parts: Vec<&str> = line.split('\t').collect();

        let range = Range::from_str(parts[0]);
        if !range.is_valid() {
            continue;
        }

        let signal = parts[2];

        let ctg_id = garr::find_one(&mut conn, range.chr(), *range.start(), *range.end());
        if ctg_id.is_empty() {
            continue;
        }

        // Redis counter
        let counter = format!("cnt:peak:{}", ctg_id);
        let serial: isize = conn.incr(counter.clone(), 1).unwrap();
        let peak_id = format!("peak:{}:{}", ctg_id, serial);

        let length = range.end() - range.start() + 1;
        let gc_content = garr::get_gc_content(&mut conn, range.chr(), *range.start(), *range.end());

        let _: () = redis::pipe()
            .hset(&peak_id, "chr_name", range.chr())
            .ignore()
            .hset(&peak_id, "chr_start", *range.start())
            .ignore()
            .hset(&peak_id, "chr_end", *range.end())
            .ignore()
            .hset(&peak_id, "length", length)
            .ignore()
            .hset(&peak_id, "gc", gc_content)
            .ignore()
            .hset(&peak_id, "signal", signal)
            .ignore()
            .query(&mut conn)
            .unwrap();
    }

    // number of peaks
    let peak_count = garr::get_scan_count(&mut conn, "peak:*".to_string());
    println!("There are {} peaks", peak_count);

    Ok(())
}
