use clap::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
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
                .num_args(1)
                .default_value("ctg:*")
                .help("Sets the pattern to scan, `ctg:*`"),
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
    let opt_pattern = args.get_one::<String>("scan").unwrap().as_str();

    // there are no easy ways to get field names of a struct
    // let Serde handles them
    let writer = intspan::writer(args.get_one::<String>("outfile").unwrap());
    let mut tsv_wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_writer(writer);

    // redis connection
    let mut conn = gams::connect();
    let mut conn2 = gams::connect(); // can't use one same `conn` inside an iter

    // scan
    let iter: redis::Iter<'_, String> = conn.scan_match(opt_pattern).unwrap();
    for id in iter {
        if opt_pattern.starts_with("ctg") {
            let value: gams::Ctg = gams::get_ctg(&mut conn2, &id);
            tsv_wtr.serialize(value).unwrap();
        }
        else if opt_pattern.starts_with("feature") {
            let bytes = gams::get_bin(&mut conn2, &id);
            let value: gams::Feature = bincode::deserialize(&bytes).unwrap();
            tsv_wtr.serialize(value).unwrap();
        }
    }

    Ok(())
}
