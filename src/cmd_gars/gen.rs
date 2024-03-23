use bio::io::fasta;
use clap::*;
use flate2::read::GzEncoder;
use flate2::Compression;
use gars::Ctg;
use intspan::{IntSpan, Range};
use redis::Commands;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::io::Read;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("gen")
        .about("Generate the database from (gzipped) fasta files")
        .after_help(
            r###"
Top level keys:
common_name => STRING
chr         => HASH

ctg:
Serial  format!("cnt:ctg:{}", chr_id)           i32
ID      format!("ctg:{}:{}", chr_id, serial)
Index   format!("idx:ctg:{}", chr_id)           Lapper<u32, String>
bincode format!("bin:ctg:{}", chr_id)           BTreeMap<String, Ctg>

seq:
ID      format!("seq:{}", ctg_id)               Gzipped &[u8]

"###,
        )
        .arg(
            Arg::new("infiles")
                .index(1)
                .num_args(1..)
                .help("Set the input files to use"),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .short('n')
                .num_args(1)
                .default_value("target")
                .help("The common name, e.g. S288c"),
        )
        .arg(
            Arg::new("piece")
                .long("piece")
                .num_args(1)
                .default_value("500000")
                .value_parser(value_parser!(i32))
                .help("Break genome into pieces"),
        )
        .arg(
            Arg::new("fill")
                .long("fill")
                .num_args(1)
                .default_value("50")
                .value_parser(value_parser!(i32))
                .help("Fill gaps smaller than this"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .num_args(1)
                .default_value("5000")
                .value_parser(value_parser!(i32))
                .help("Skip pieces smaller than this"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let common_name = args.get_one::<String>("name").unwrap().as_str();
    let piece = *args.get_one::<i32>("piece").unwrap();
    let fill = *args.get_one::<i32>("fill").unwrap();
    let min = *args.get_one::<i32>("min").unwrap();

    // redis connection
    let mut conn = gars::connect();

    // common_name
    let _: () = conn.set("common_name", common_name).unwrap();

    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = intspan::reader(infile);
        let fa_in = fasta::Reader::new(reader);

        for result in fa_in.records() {
            // obtain record or fail with error
            let record = result.unwrap();

            let chr_id = record.id();
            let chr_seq = record.seq();

            // hash chr
            let _: () = conn.hset("chr", chr_id, chr_seq.len()).unwrap();

            // ([start, end], [start, end], ...)
            let mut regions = VecDeque::new();
            {
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
                eprintln!(
                    "Ambiguous region for {}:\n{}\n",
                    chr_id,
                    ambiguous_set.runlist()
                );

                let mut valid_set = IntSpan::new();
                valid_set.add_pair(1, chr_seq.len() as i32);
                valid_set.subtract(&ambiguous_set);
                valid_set = valid_set.fill(fill - 1);
                valid_set = valid_set.excise(min);
                eprintln!("Valid region for {}:\n{}\n", chr_id, valid_set.runlist());

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
            }

            // ctgs in each chr
            let mut ctgs: BTreeMap<String, Ctg> = BTreeMap::new();
            while !regions.is_empty() {
                // Redis counter
                let serial: isize = conn.incr(format!("cnt:ctg:{}", chr_id), 1).unwrap();
                let ctg_id = format!("ctg:{}:{}", chr_id, serial);

                let start = regions.pop_front().unwrap();
                let end = regions.pop_front().unwrap();

                let range = Range::from(&chr_id, start, end);

                // ID	chr_id	chr_start	chr_end	chr_strand	length
                let ctg = gars::Ctg {
                    id: ctg_id.clone(),
                    range: range.to_string(),
                    chr_id: chr_id.to_string(),
                    chr_start: start,
                    chr_end: end,
                    chr_strand: "+".to_string(),
                    length: end - start + 1,
                };
                ctgs.insert(ctg_id.clone(), ctg);

                let _: () = redis::pipe()
                    .hset(&ctg_id, "chr_id", chr_id)
                    .ignore()
                    .hset(&ctg_id, "chr_start", start)
                    .ignore()
                    .hset(&ctg_id, "chr_end", end)
                    .ignore()
                    .hset(&ctg_id, "chr_strand", "+")
                    .ignore()
                    .hset(&ctg_id, "length", end - start + 1)
                    .ignore()
                    .query(&mut conn)
                    .unwrap();

                let seq: &[u8] = &chr_seq[(start - 1) as usize..end as usize];
                let gz_seq = encode_gz(seq).unwrap();
                let _: () = conn.set(format!("seq:{}", ctg_id), &gz_seq).unwrap();

                // zrange indexing ctg
                let _: () = conn
                    .zadd(format!("ctg-s:{}", chr_id), &ctg_id, start)
                    .unwrap();
                let _: () = conn
                    .zadd(format!("ctg-e:{}", chr_id), &ctg_id, end)
                    .unwrap();
            } // ctg

            let ctgs_bytes = bincode::serialize(&ctgs).unwrap();
            let _: () = conn
                .set(format!("bin:ctg:{}", chr_id), &ctgs_bytes)
                .unwrap();
        } // chr
    } // fasta file

    eprintln!("Building the lapper index of ctgs...\n");
    gars::build_idx_ctg(&mut conn);

    // number of chr
    let n_chr: i32 = conn.hlen("chr").unwrap();
    eprintln!("There are {} chromosomes", n_chr);

    // number of ctg
    let n_ctg: i32 = gars::get_scan_count(&mut conn, "ctg:*");
    eprintln!("There are {} contigs", n_ctg);

    Ok(())
}

fn encode_gz(seq: &[u8]) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut z = GzEncoder::new(seq, Compression::fast());
    z.read_to_end(&mut bytes)?;
    Ok(bytes)
}
