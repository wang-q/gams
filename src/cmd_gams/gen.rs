use bio::io::fasta;
use clap::*;
use std::collections::{BTreeMap, VecDeque};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("gen")
        .about("Generate the database from (gzipped) fasta files")
        .after_help(
            r###"
top:
    top:common_name => STRING
    top:chrs        => Vec<String>
    top:chr_len     => BTreeMap<String, i32>

ctg:
    cnt:ctg:{chr_id}        => serial
    ctg:{chr_id}:{serial}   => Ctg
    idx:ctg:{chr_id}        => index, Lapper<u32, String>
    bundle:ctg:{chr_id}     => BTreeMap<ctg_id, Ctg>
                               all contigs of a chr_id

seq:
    seq:{ctg_id}            => Gzipped &[u8]

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
    let opt_name = args.get_one::<String>("name").unwrap().as_str();
    let opt_piece = *args.get_one::<i32>("piece").unwrap();
    let opt_fill = *args.get_one::<i32>("fill").unwrap();
    let opt_min = *args.get_one::<i32>("min").unwrap();

    // redis connection
    let mut conn = gams::connect();

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
                let serial = gams::incr_serial(&mut conn, &format!("cnt:ctg:{chr_id}"));
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

                gams::insert_ctg(&mut conn, &ctg_id, &ctg);

                let seq: &[u8] = &chr_seq[(start - 1) as usize..end as usize];
                gams::insert_seq(&mut conn, &ctg_id, seq);
            } // ctg

            let ctgs_bytes = bincode::serialize(&ctg_of).unwrap();
            gams::insert_bin(&mut conn, &format!("bundle:ctg:{chr_id}"), &ctgs_bytes);
        } // chr
    } // fasta file

    // store to db
    {
        // common_name
        gams::insert_str(&mut conn, "top:common_name", opt_name);

        // chrs
        let bin_chr_len = bincode::serialize(&len_of).unwrap();
        gams::insert_bin(&mut conn, "top:chr_len", &bin_chr_len);

        let bin_chrs = bincode::serialize(&len_of.keys().cloned().collect::<Vec<_>>()).unwrap();
        gams::insert_bin(&mut conn, "top:chrs", &bin_chrs);

        eprintln!("Building the lapper index of ctgs...\n");
        gams::build_idx_ctg(&mut conn);
    }

    {
        let common_name = gams::get_str(&mut conn, "top:common_name");
        eprintln!("Common name: {}", common_name);

        // number of chr
        let n_chr = gams::get_vec_chr(&mut conn).len();
        eprintln!("There are {} chromosomes", n_chr);

        // number of ctg
        let n_ctg: i32 = gams::get_scan_count(&mut conn, "ctg:*");
        eprintln!("There are {} contigs", n_ctg);
    }

    Ok(())
}
