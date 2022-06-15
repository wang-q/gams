use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("tsv")
        .about("Export Redis hashes to a tsv file")
        .after_help(
            r#"
All hashes should have the same structure.
ID, chr_id, chr_start, chr_end will always be included.

"#,
        )
        .arg(
            Arg::new("scan")
                .long("scan")
                .short('s')
                .takes_value(true)
                .default_value("ctg:*")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Sets the pattern to scan, `ctg:*`"),
        )
        .arg(
            Arg::new("field")
                .long("field")
                .short('f')
                .action(ArgAction::Append)
                .takes_value(true)
                .help("Sets the fields to output"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .short('r')
                .takes_value(false)
                .help("Write a `range` field before the chr_id field"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    //----------------------------
    // Options
    //----------------------------
    let pattern = args.get_one::<String>("scan").unwrap().as_str();
    let fields: Vec<String> = if args.contains_id("field") {
        args.get_many::<String>("field")
            .unwrap()
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };
    let is_range = args.contains_id("range");

    // redis connection
    let mut conn = connect();
    let mut conn2 = connect(); // can't use one same `conn` inside an iter

    // headers
    let mut writer = writer(args.get_one::<String>("outfile").unwrap());

    // scan
    let mut headers: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match(pattern).unwrap();
    for id in iter {
        // need headers
        if headers.is_empty() {
            let mut keys: Vec<String> = conn2.hkeys(&id).unwrap();
            for k in ["chr_id", "chr_start", "chr_end"]
                .iter()
                .map(|s| s.to_string())
            {
                if keys.contains(&k) {
                    headers.push(k.clone());

                    let index = keys.iter().position(|e| *e == k).unwrap();
                    keys.remove(index);
                }
            }

            if fields.is_empty() {
                for k in keys {
                    headers.push(k.clone());
                }
            } else {
                for k in &fields {
                    if keys.contains(k) {
                        headers.push(k.clone());
                    }
                }
            }

            let mut header_line = headers.join("\t");
            if is_range {
                header_line = header_line.replace("chr_id\t", "range\tchr_id\t");
            }
            writer.write_all(format!("{}\t{}\n", "ID", header_line).as_ref())?;
        }

        //----------------------------
        // Output
        //----------------------------
        let hash: BTreeMap<String, String> = conn2.hgetall(&id).unwrap();
        let mut values: Vec<String> = Vec::new();
        if !is_range {
            for k in &headers {
                let val = hash.get(k).unwrap();
                values.push(val.clone());
            }
        } else {
            let (chr_id, chr_start, chr_end) = gars::get_key_pos(&mut conn2, &id);
            let range = Range::from(&chr_id, chr_start, chr_end);
            values.push(range.to_string());

            for k in &headers {
                let val = hash.get(k).unwrap();
                values.push(val.clone());
            }
        }

        let line = values.join("\t");
        writer.write_all(format!("{}\t{}\n", id, line).as_ref())?;
    }

    Ok(())
}
