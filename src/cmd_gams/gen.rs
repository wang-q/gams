use bio::io::fasta;
use clap::*;
use std::collections::{BTreeMap, VecDeque};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("gen")
        .about("Generate the database from (gzipped) fasta files")
        .after_help(
            r###"
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
                .help("Divide genome into pieces"),
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
    let opt_name = args.get_one::<String>("name").unwrap().as_str();
    let opt_piece = *args.get_one::<i32>("piece").unwrap();
    let opt_fill = *args.get_one::<i32>("fill").unwrap();
    let opt_min = *args.get_one::<i32>("min").unwrap();

    // redis connection
    let mut conn = gams::Conn::new();

    // hash chr
    let mut len_of: BTreeMap<_, _> = BTreeMap::new();

    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = intspan::reader(infile);
        let fa_in = fasta::Reader::new(reader);

        for result in fa_in.records() {
            // obtain record or fail with error
            let record = result.unwrap();

            let chr_id = record.id();
            let chr_seq = record.seq();

            len_of.insert(chr_id.to_string(), chr_seq.len());

            // ([start, end], [start, end], ...)
            let mut regions = VecDeque::new();
            {
                // Ambiguous region
                let mut ambiguous_set = intspan::IntSpan::new();

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

                let mut valid_set = intspan::IntSpan::new();
                valid_set.add_pair(1, chr_seq.len() as i32);
                valid_set.subtract(&ambiguous_set);
                valid_set = valid_set.fill(opt_fill - 1);
                valid_set = valid_set.excise(opt_min);
                eprintln!("Valid region for {}:\n{}\n", chr_id, valid_set.runlist());

                let valid_ranges = valid_set.ranges();
                for i in 0..valid_set.span_size() {
                    let mut cur_regions = vec![];
                    let mut pos = *valid_ranges.get(i * 2).unwrap();
                    let max = *valid_ranges.get(i * 2 + 1).unwrap();
                    while max - pos + 1 > opt_piece {
                        cur_regions.push(pos);
                        cur_regions.push(pos + opt_piece - 1);
                        pos += opt_piece;
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

            // ctgs of each chr
            let mut ctg_of: BTreeMap<String, gams::Ctg> = BTreeMap::new();
            while !regions.is_empty() {
                // Redis counter
                let serial = conn.incr_sn(&format!("cnt:ctg:{chr_id}"));
                let ctg_id = format!("ctg:{chr_id}:{serial}");

                let start = regions.pop_front().unwrap();
                let end = regions.pop_front().unwrap();

                let range = intspan::Range::from(chr_id, start, end);

                // ID	chr_id	chr_start	chr_end	chr_strand	length
                let ctg = gams::Ctg {
                    id: ctg_id.clone(),
                    range: range.to_string(),
                    chr_id: chr_id.to_string(),
                    chr_start: start,
                    chr_end: end,
                    chr_strand: "+".to_string(),
                    length: end - start + 1,
                };
                ctg_of.insert(ctg_id.clone(), ctg.clone());

                conn.insert_ctg(&ctg_id, &ctg);

                let seq: &[u8] = &chr_seq[(start - 1) as usize..end as usize];
                conn.insert_seq(&ctg_id, seq);
            } // ctg

            let bundle_ctgs = bincode::serialize(&ctg_of).unwrap();
            conn.insert_bin(&format!("bundle:ctg:{chr_id}"), &bundle_ctgs);
        } // chr
    } // fasta file

    // store to db
    {
        // common_name
        conn.insert_str("top:common_name", opt_name);

        // chrs
        let json_chr_len = serde_json::to_string(&len_of).unwrap();
        conn.insert_str("top:chr_len", &json_chr_len);

        let json_chrs = serde_json::to_string(&len_of.keys().cloned().collect::<Vec<_>>()).unwrap();
        conn.insert_str("top:chrs", &json_chrs);

        eprintln!("Building the index of ctgs...\n");
        conn.build_idx_ctg();
    }

    {
        let common_name = conn.get_str("top:common_name");
        eprintln!("Common name: {}", common_name);

        // number of chr
        let n_chr = conn.get_vec_chr().len();
        eprintln!("There are {} chromosomes", n_chr);

        // number of ctg
        let n_ctg: i32 = conn.get_scan_count("ctg:*");
        eprintln!("There are {} ctgs", n_ctg);
    }

    Ok(())
}
