extern crate clap;

use clap::*;
use intspan::writer;
use polars::prelude::*;

fn main() -> std::io::Result<()> {
    let app = Command::new("gars-stat")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Build-in stats for gars")
        .arg_required_else_help(true)
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
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Output filename. [stdout] for screen"),
        );

    let args = app.get_matches();

    execute(&args).unwrap();

    Ok(())
}

// command implementation
fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // opts
    let infile = args.get_one::<String>("infile").unwrap();
    let query = args.get_one::<String>("query").unwrap().as_str();

    let writer = writer(args.get_one::<String>("outfile").unwrap());

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
        .groupby(&["chr_id"])
        .unwrap()
        .agg(&[("length", &["count", "mean"])]);

    res.unwrap()
}
