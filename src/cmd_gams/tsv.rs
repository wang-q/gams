use clap::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("tsv")
        .about("Export Redis hashes to a tsv file")
        .after_help(
            r###"
"###,
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
    let mut raw_conn = gams::connect();
    let mut conn = gams::Conn::new(); // can't use one same `conn` inside an iter

    // scan
    let iter: redis::Iter<'_, String> = raw_conn.scan_match(opt_pattern).unwrap();
    for id in iter {
        if opt_pattern.starts_with("ctg") {
            let value: gams::Ctg = conn.get_ctg(&id);
            tsv_wtr.serialize(value).unwrap();
        } else if opt_pattern.starts_with("feature") {
            let json = conn.get_str(&id);
            let value: gams::Feature = serde_json::from_str(&json).unwrap();
            tsv_wtr.serialize(value).unwrap();
        } else if opt_pattern.starts_with("rg") {
            let json = conn.get_str(&id);
            let value: gams::Rg = serde_json::from_str(&json).unwrap();
            tsv_wtr.serialize(value).unwrap();
        } else if opt_pattern.starts_with("peak") {
            let json = conn.get_str(&id);
            let value: gams::Peak = serde_json::from_str(&json).unwrap();
            tsv_wtr.serialize(value).unwrap();
        }
    }

    Ok(())
}
