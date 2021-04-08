use bio::io::fasta;
use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("gen")
        .about("Generate the database from (gzipped) fasta files")
        .arg(
            Arg::with_name("infiles")
                .help("Sets the input files to use")
                .required(true)
                .min_values(1)
                .index(1),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .default_value("target")
                .empty_values(false)
                .help("The common name, e.g. S288c"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // redis connection
    let mut conn = connect();

    // common_name
    let _: () = conn
        .set("common_name", args.value_of("name").unwrap())
        .unwrap();

    for infile in args.values_of("infiles").unwrap() {
        let reader = reader(infile);
        let fa_in = fasta::Reader::new(reader);

        for result in fa_in.records() {
            // obtain record or fail with error
            let record = result.unwrap();

            let chr_id = record.id();
            let chr_seq = record.seq();

            // hash chr
            let _: () = conn.hset("chr", chr_id, chr_seq.len()).unwrap();

            // contigs in each chr
            let ctg_id = format!("ctg:{}:1", chr_id);
            let _: () = conn.hset(&ctg_id, "chr_name", chr_id).unwrap();
            let _: () = conn.hset(&ctg_id, "chr_start", 1).unwrap();
            let _: () = conn.hset(&ctg_id, "chr_end", chr_seq.len()).unwrap();
            let _: () = conn.hset(&ctg_id, "chr_strand", "+").unwrap();
            let _: () = conn.hset(&ctg_id, "length", chr_seq.len()).unwrap();
            let _: () = conn.hset(&ctg_id, "seq", chr_seq).unwrap();

            // indexing ctg
            let _: () = conn.zadd(format!("ctg-s:{}", chr_id), &ctg_id, 1).unwrap();
            let _: () = conn
                .zadd(format!("ctg-e:{}", chr_id), &ctg_id, chr_seq.len())
                .unwrap();
        }
    }

    // number of chr
    let n_chr: i32 = conn.hlen("chr").unwrap();
    println!("There are {} chromosomes", n_chr);

    Ok(())
}
