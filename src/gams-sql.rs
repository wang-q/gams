extern crate clap;

use clap::*;
use datafusion::arrow::csv;
use datafusion::prelude::*;
use intspan::writer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let app = Command::new("gams-sql")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Build-in stats for gams")
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .arg(
            Arg::new("infile")
                .index(1)
                .num_args(1)
                .help("Sets the input file to use"),
        )
        .arg(
            Arg::new("query")
                .index(2)
                .num_args(1)
                .help("SQL query file"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        );

    let args = app.get_matches();

    execute(&args).await?;

    Ok(())
}

// command implementation
async fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let infile = args.get_one::<String>("infile").unwrap();
    let query_file = args.get_one::<String>("query").unwrap();
    let query = std::fs::read_to_string(query_file).unwrap();

    let writer = writer(args.get_one::<String>("outfile").unwrap());

    // Initialize query interface
    let ctx = SessionContext::new();

    // Register data sources
    let options = CsvReadOptions::new()
        .file_extension(".tsv")
        .has_header(true)
        .delimiter(b'\t');
    ctx.register_csv("ctg", infile, options).await?;

    // create a plan
    let df = ctx.sql(&query).await?;

    // execute the plan
    // TODO: await makes the compilation extremely slow
    let results = df.collect().await?;

    // eprintln!("{:?}", results);

    // create a builder and writer
    let builder = csv::WriterBuilder::new()
        .with_header(true)
        .with_delimiter(b'\t');
    let mut csv_writer = builder.build(writer);
    for res in results {
        csv_writer.write(&res).unwrap();
    }

    Ok(())
}
