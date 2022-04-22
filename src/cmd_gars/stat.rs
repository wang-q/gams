use clap::*;
use intspan::writer;
use polars::prelude::*;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("stat")
        .about("Build-in stats")
        .arg(
            Arg::new("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("query")
                .default_value("ctg")
                .help("Query name")
                .required(true)
                .index(2),
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
    let infile = args.value_of("infile").unwrap();
    let query = args.value_of("query").unwrap();

    let writer = writer(args.value_of("outfile").unwrap());

    let df = CsvReader::from_path(infile)
        .unwrap()
        .infer_schema(None)
        .has_header(true)
        .with_delimiter(b'\t')
        .finish()
        .unwrap();

    let mut res = match query {
        "ctg" => query_ctg(df),
        _ => unreachable!(),
    };

    // write DataFrame to file
    CsvWriter::new(writer)
        .has_header(true)
        .with_delimiter(b'\t')
        .finish(&mut res)
        .unwrap();

    Ok(())
}

fn query_ctg(df: DataFrame) -> DataFrame {
    let res = df
        .groupby(&["chr_name"])
        .unwrap()
        .agg(&[("length", &["count", "mean"])]);

    res.unwrap()
}
