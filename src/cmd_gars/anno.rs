use std::collections::HashMap;
use clap::*;
use gars::*;
use intspan::*;
use lazy_static::lazy_static;
use redis::Commands;
use regex::Regex;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::Path;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
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
                .help("Set the runlist file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("infiles")
                .help("Set the input files to use")
                .required(true)
                .index(2)
                .min_values(1),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .takes_value(false)
                .help("Treat the first line of each file as a header"),
        )
        .arg(
            Arg::new("id")
                .long("id")
                .takes_value(true)
                .default_value("1")
                .required(true)
                .help("Set the index of the ID field"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .takes_value(true)
                .default_value("2")
                .help("Set the index of the range field"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .forbid_empty_values(true)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    //----------------------------
    // Options
    //----------------------------
    let mut writer = writer(args.value_of("outfile").unwrap());

    let is_header = args.is_present("header");

    let idx_id: usize = args.value_of_t("id").unwrap_or_else(|e| {
        eprintln!("Need an integer for --id\n{}", e);
        std::process::exit(1)
    });

    let idx_range: usize = args.value_of_t("range").unwrap_or_else(|e| {
        eprintln!("Need an integer for --range\n{}", e);
        std::process::exit(1)
    });

    //----------------------------
    // Loading
    //----------------------------
    let yaml = read_yaml(args.value_of("runlist").unwrap());
    let set = yaml2set(&yaml);

    // local caches of the feature runlist for each ctg
    let mut cache: HashMap<String, String> = HashMap::new();

    //----------------------------
    // Operating
    //----------------------------
    for infile in args.values_of("infiles").unwrap() {
        let reader = reader(infile);
        'LINE: for (i, line) in reader.lines().filter_map(|r| r.ok()).enumerate() {
            if is_header && i == 0 {
                let prefix = Path::new(args.value_of("runlist").unwrap())
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

            let field_id = parts.get(idx_id - 1).unwrap();
            let ctg_id = match extract_ctg_id(field_id) {
                Some(ctg_id) => ctg_id,
                None => continue,
            };

            let range = Range::from_str(parts.get(idx_range - 1).unwrap());

            if !range.is_valid() {
                continue 'LINE;
            }

            let mut intspan = range.intspan();

            eprintln!("ctg_id = {:#?}", ctg_id);
            eprintln!("range = {:#?}", range.to_string());
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
