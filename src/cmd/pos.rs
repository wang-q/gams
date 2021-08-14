use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;
use std::collections::HashMap;
use std::io::BufRead;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pos")
        .about("Add position files to pos")
        .arg(
            Arg::with_name("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("tag")
                .long("tag")
                .short("t")
                .takes_value(true)
                .default_value("pos")
                .empty_values(false)
                .help("Position tags"),
        )
        .arg(
            Arg::with_name("style")
                .long("style")
                .takes_value(true)
                .default_value("intact")
                .empty_values(false)
                .help("Style of sliding windows, intact or center"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let infile = args.value_of("infile").unwrap();
    let tag = args.value_of("tag").unwrap();

    // redis connection
    let mut conn = connect();

    // positions in each contig
    let mut pos_serial: HashMap<String, i32> = HashMap::new();

    // processing each line
    let reader = reader(infile);
    for line in reader.lines().filter_map(|r| r.ok()) {
        let range = Range::from_str(&line);
        if !range.is_valid() {
            continue;
        }

        let ctg_id = garr::find_one(&mut conn, range.chr(), *range.start(), *range.end());
        if ctg_id == "" {
            continue;
        }

        let serial = pos_serial.entry(ctg_id.clone()).or_insert(0);
        *serial += 1;

        let pos_id = format!("pos:{}:{}", ctg_id, serial);

        let length = range.end() - range.start() + 1;
        let _: () = conn.hset(&pos_id, "length", length).unwrap();

        let _: () = conn.hset(&pos_id, "chr_name", range.chr()).unwrap();
        let _: () = conn.hset(&pos_id, "chr_start", *range.start()).unwrap();
        let _: () = conn.hset(&pos_id, "chr_end", *range.end()).unwrap();

        let gc_content = garr::get_gc_content(&mut conn, range.chr(), *range.start(), *range.end());
        let _: () = conn.hset(&pos_id, "gc", gc_content).unwrap();

        let _: () = conn.hset(&pos_id, "tag", tag).unwrap();

        // let start = regions.pop_front().unwrap() as usize;
        // let end = regions.pop_front().unwrap() as usize;
        //
        // set.entry(chr.to_string())
        //     .and_modify(|e| e.add_pair(range.start().clone(), range.end().clone()));
    }

    // number of pos
    let pos_count = garr::get_scan_count(&mut conn, "pos:*".to_string());
    println!("There are {} positions", pos_count);

    Ok(())
}
