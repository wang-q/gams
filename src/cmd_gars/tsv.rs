use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("tsv")
        .about("Exports Redis hashes to a tsv file")
        .after_help(
            "\
    All hashes should have the same structure. \
    ID, chr_name, chr_start, chr_end will always be included. \
             ",
        )
        .arg(
            Arg::new("scan")
                .long("scan")
                .short('s')
                .takes_value(true)
                .default_value("ctg:*")
                .forbid_empty_values(true)
                .help("Sets the pattern to scan, `ctg:*`"),
        )
        .arg(
            Arg::new("field")
                .long("field")
                .short('f')
                .multiple_values(true)
                .takes_value(true)
                .help("Sets the fields to output"),
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
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let pattern = args.value_of("scan").unwrap();
    let fields: Vec<String> = if args.is_present("field") {
        args.values_of("field")
            .unwrap()
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    // redis connection
    let mut conn = connect();
    let mut conn2 = connect(); // can't use one same `conn` inside an iter

    // headers
    let mut writer = writer(args.value_of("outfile").unwrap());

    // scan
    let mut headers: Vec<String> = Vec::new();
    let iter: redis::Iter<'_, String> = conn.scan_match(pattern).unwrap();
    for x in iter {
        // need headers
        if headers.is_empty() {
            let mut keys: Vec<String> = conn2.hkeys(&x).unwrap();
            for k in ["chr_name", "chr_start", "chr_end"]
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
            let line = headers.join("\t");
            writer.write_all(format!("{}\t{}\n", "ID", line).as_ref())?;
        }

        let hash: BTreeMap<String, String> = conn2.hgetall(&x).unwrap();
        let mut values: Vec<String> = Vec::new();
        for k in &headers {
            let val = hash.get(k).unwrap();
            values.push(val.clone());
        }
        let line = values.join("\t");
        writer.write_all(format!("{}\t{}\n", x, line).as_ref())?;
    }

    Ok(())
}
