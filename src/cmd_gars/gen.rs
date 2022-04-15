use crate::*;
use bio::io::fasta;
use clap::*;
use intspan::*;
use redis::Commands;
use std::collections::VecDeque;
use std::iter::FromIterator;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("gen")
        .about("Generate the database from (gzipped) fasta files")
        .arg(
            Arg::new("infiles")
                .help("Sets the input files to use")
                .required(true)
                .min_values(1)
                .index(1),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .short('n')
                .takes_value(true)
                .default_value("target")
                .forbid_empty_values(true)
                .help("The common name, e.g. S288c"),
        )
        .arg(
            Arg::new("piece")
                .long("piece")
                .takes_value(true)
                .default_value("500000")
                .forbid_empty_values(true)
                .help("Break genome into pieces"),
        )
        .arg(
            Arg::new("fill")
                .long("fill")
                .takes_value(true)
                .default_value("50")
                .forbid_empty_values(true)
                .help("Fill gaps smaller than this"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .takes_value(true)
                .default_value("5000")
                .forbid_empty_values(true)
                .help("Skip pieces smaller than this"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let common_name = args.value_of("name").unwrap();
    let piece: i32 = args.value_of_t("piece").unwrap_or_else(|e| {
        eprintln!("Need a integer for --piece\n{}", e);
        std::process::exit(1)
    });

    let fill: i32 = args.value_of_t("fill").unwrap_or_else(|e| {
        eprintln!("Need a integer for --fill\n{}", e);
        std::process::exit(1)
    });

    let min: i32 = args.value_of_t("min").unwrap_or_else(|e| {
        eprintln!("Need a integer for --min\n{}", e);
        std::process::exit(1)
    });

    // redis connection
    let mut conn = crate::connect();

    // common_name
    let _: () = conn.set("common_name", common_name).unwrap();

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

            // Ambiguous region
            let mut ambiguous_set = IntSpan::new();

            for (i, item) in chr_seq.iter().enumerate() {
                match *item as char {
                    'A' | 'C' | 'G' | 'T' | 'a' | 'c' | 'g' | 't' => {}
                    _ => {
                        ambiguous_set.add_n(i as i32 + 1);
                    }
                }
            }
            println!(
                "Ambiguous region for {}:\n    {}\n",
                chr_id,
                ambiguous_set.runlist()
            );

            let mut valid_set = IntSpan::new();
            valid_set.add_pair(1, chr_seq.len() as i32);
            valid_set.subtract(&ambiguous_set);
            valid_set = valid_set.fill(fill - 1);
            valid_set = valid_set.excise(min);
            println!(
                "Valid region for {}:\n    {}\n",
                chr_id,
                valid_set.runlist()
            );

            // ([start, end], [start, end], ...)
            let mut regions = vec![];
            let valid_ranges = valid_set.ranges();
            for i in 0..valid_set.span_size() {
                let mut cur_regions = vec![];
                let mut pos = *valid_ranges.get(i * 2).unwrap();
                let max = *valid_ranges.get(i * 2 + 1).unwrap();
                while max - pos + 1 > piece {
                    cur_regions.push(pos);
                    cur_regions.push(pos + piece - 1);
                    pos += piece;
                }

                if cur_regions.is_empty() {
                    cur_regions.push(pos);
                    cur_regions.push(max);
                } else if let Some(last) = cur_regions.last_mut() {
                    *last = max;
                }

                regions.extend(cur_regions);
            }
            let mut regions = VecDeque::from_iter(regions);

            // contigs in each chr
            let mut ctg_serial = 1;
            while !regions.is_empty() {
                let ctg_id = format!("ctg:{}:{}", chr_id, ctg_serial);
                let start = regions.pop_front().unwrap() as usize;
                let end = regions.pop_front().unwrap() as usize;

                let _: () = conn.hset(&ctg_id, "chr_name", chr_id).unwrap();
                let _: () = conn.hset(&ctg_id, "chr_start", start).unwrap();
                let _: () = conn.hset(&ctg_id, "chr_end", end).unwrap();
                let _: () = conn.hset(&ctg_id, "chr_strand", "+").unwrap();
                let _: () = conn.hset(&ctg_id, "length", end - start + 1).unwrap();
                let _: () = conn
                    .set(format!("seq:{}", ctg_id), &chr_seq[start - 1..end])
                    .unwrap();

                // indexing ctg
                let _: () = conn
                    .zadd(format!("ctg-s:{}", chr_id), &ctg_id, start)
                    .unwrap();
                let _: () = conn
                    .zadd(format!("ctg-e:{}", chr_id), &ctg_id, end)
                    .unwrap();

                ctg_serial += 1;
            }
        }
    }

    // number of chr
    let n_chr: i32 = conn.hlen("chr").unwrap();
    println!("There are {} chromosomes", n_chr);

    // number of ctg
    let n_ctg: i32 = crate::get_scan_count(&mut conn, "ctg:*".to_string());
    println!("There are {} contigs", n_ctg);

    Ok(())
}
