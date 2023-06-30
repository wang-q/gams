use clap::*;
use gars::*;
use intspan::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::Path;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("anno")
        .about("Annotate anything that contains a ctg_id and a range")
        .after_help(
            r###"
* This command is a simplified and accelerated version of `rgr prop`
* Lines without a valid ctg_id and a valid range will not be output
* If `--header` is set, the appended field name will be `prefixProp`

"###,
        )
        .arg(
            Arg::new("runlist")
                .required(true)
                .index(1)
                .num_args(1)
                .help("Set the runlist file to use"),
        )
        .arg(
            Arg::new("infiles")
                .required(true)
                .index(2)
                .num_args(1..)
                .help("Set the input files to use"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each file as a header"),
        )
        .arg(
            Arg::new("id")
                .long("id")
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("Set the index of the ID field"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .num_args(1)
                .default_value("2")
                .value_parser(value_parser!(usize))
                .help("Set the index of the range field"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    //----------------------------
    // Options
    //----------------------------
    let mut writer = writer(args.get_one::<String>("outfile").unwrap());

    let is_header = args.get_flag("header");
    let idx_id = *args.get_one::<usize>("id").unwrap();
    let idx_range = *args.get_one::<usize>("range").unwrap();

    //----------------------------
    // Loading
    //----------------------------
    // redis connection
    let mut conn = connect();

    let yaml = read_yaml(args.get_one::<String>("runlist").unwrap());
    let set = yaml2set(&yaml);

    // local caches of the feature IntSpan for each ctg
    let mut cache: HashMap<String, IntSpan> = HashMap::new();

    //----------------------------
    // Operating
    //----------------------------
    for infile in args.get_many::<String>("infiles").unwrap() {
        let reader = reader(infile);
        'LINE: for (i, line) in reader.lines().map_while(Result::ok).enumerate() {
            if is_header && i == 0 {
                let prefix = Path::new(args.get_one::<String>("runlist").unwrap())
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .split('.')
                    .next()
                    .unwrap()
                    .to_string();

                writer.write_fmt(format_args!("{}\t{}{}\n", line, prefix, "Prop"))?;

                continue 'LINE;
            }

            let parts: Vec<&str> = line.split('\t').collect();

            let line_id = parts.get(idx_id - 1).unwrap();
            let ctg_id = match extract_ctg_id(line_id) {
                Some(ctg_id) => ctg_id,
                None => continue,
            }
            .to_string();

            let range = Range::from_str(parts.get(idx_range - 1).unwrap());
            if !range.is_valid() {
                continue 'LINE;
            }
            let intspan = range.intspan();

            let mut prop = 0.0;
            if set.contains_key(range.chr()) {
                if !cache.contains_key(&ctg_id) {
                    let (_, chr_start, chr_end) = gars::get_key_pos(&mut conn, &ctg_id);
                    let ctg_intspan = IntSpan::from_pair(chr_start, chr_end);
                    let parent = set.get(range.chr()).unwrap().intersect(&ctg_intspan);
                    cache.insert(ctg_id.clone(), parent);
                }

                let intxn = cache.get(&ctg_id).unwrap().intersect(&intspan);
                prop = intxn.cardinality() as f32 / intspan.cardinality() as f32;
            }
            writer.write_fmt(format_args!("{}\t{:.4}\n", line, prop))?;
        }
    }

    Ok(())
}

fn extract_ctg_id(input: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?xi)
            (?P<ctg>ctg:[\w_]+:\d+)
            "
        )
        .unwrap();
    }
    RE.captures(input)
        .and_then(|cap| cap.name("ctg").map(|ctg| ctg.as_str()))
}
