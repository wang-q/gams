use clap::*;
use datafusion::arrow::csv;
use datafusion::arrow::datatypes::*;
use datafusion::prelude::*;
use intspan::writer;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stat")
        .about("Add range files to positions")
        .arg(
            Arg::with_name("infile")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("sql")
                .long("sql")
                .short("s")
                .takes_value(true)
                .empty_values(false)
                .help("SQL query file"),
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
#[tokio::main]
pub async fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    // opts
    let infile = args.value_of("infile").unwrap();
    let sql_file = args.value_of("sql").unwrap();
    let sql = std::fs::read_to_string(sql_file).expect("failed to read query");

    let writer = writer(args.value_of("outfile").unwrap());

    // Initialize query interface
    let mut ctx = ExecutionContext::new();

    // Register data sources
    let schema = ctg_schema();
    let options = CsvReadOptions::new()
        .schema(&schema)
        .file_extension(".tsv")
        .has_header(true)
        .delimiter(b'\t');
    ctx.register_csv("ctg", infile, options).unwrap();

    // create a plan
    let df = ctx.sql(&sql).unwrap();

    // execute the plan
    // TODO: await makes the compilation extremely slow
    let results = df.collect().await.unwrap();

    // eprintln!("{:?}", results);

    // create a builder and writer
    let builder = csv::WriterBuilder::new()
        .has_headers(true)
        .with_delimiter(b'\t');
    let mut csv_writer = builder.build(writer);
    csv_writer.write(&results[0]).unwrap();

    Ok(())
}

fn ctg_schema() -> Schema {
    Schema::new(vec![
        Field::new("ID", DataType::Utf8, true),
        Field::new("chr_name", DataType::Utf8, true),
        Field::new("chr_start", DataType::Int32, true),
        Field::new("chr_end", DataType::Int32, true),
        Field::new("chr_strand", DataType::Utf8, true),
        Field::new("length", DataType::Int32, true),
    ])
}
