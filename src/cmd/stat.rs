use clap::*;
use intspan::writer;
use polars::prelude::*;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stat")
        .about("Build-in stats")
        .arg(
            Arg::with_name("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("query")
                .default_value("ctg")
                .help("Query name")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .empty_values(false)
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

    let res = match query {
        "ctg" => query_ctg(df),
        _ => unreachable!(),
    };

    // write DataFrame to file
    CsvWriter::new(writer)
        .has_header(true)
        .with_delimiter(b'\t')
        .finish(&res);

    Ok(())
}

fn query_ctg(df: DataFrame) -> DataFrame {
    let res = df
        .groupby(&["chr_name"])
        .unwrap()
        .agg(&[("length", &["count", "mean"])]);

    res.unwrap()
}
