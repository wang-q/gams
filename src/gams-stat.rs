extern crate clap;

use clap::*;
use polars::prelude::*;

fn main() -> anyhow::Result<()> {
    let app = Command::new("gams-stat")
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
                .default_value("ctg")
                .index(2)
                .num_args(1)
                .help("Query name"),
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

    execute(&args).unwrap();

    Ok(())
}

// command implementation
fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let infile = args.get_one::<String>("infile").unwrap();
    let query = args.get_one::<String>("query").unwrap().as_str();

    let writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_separator(b'\t'))
        .try_into_reader_with_file_path(Some(infile.into()))?
        .finish()?;

    let mut res = match query {
        "ctg" => query_ctg(df),
        _ => {
            // table name in SQL
            let table = match infile {
                s if s.contains("ctg.") => "ctg",
                _ => unreachable!(),
            };
            query_sql(df, query, table)
        } // treat inputs as sql
    };

    // write DataFrame to file
    CsvWriter::new(writer)
        .with_separator(b'\t')
        .finish(&mut res)
        .unwrap();

    Ok(())
}

fn query_ctg(df: DataFrame) -> DataFrame {
    let res = df
        .lazy()
        .group_by(["chr_id"])
        .agg([
            col("id").count().alias("COUNT"),
            col("length").mean().alias("length_mean"),
            col("length").sum().alias("length_sum"),
        ])
        .sort(
            ["chr_id"],
            SortMultipleOptions::new().with_order_descending_multi([false]),
        )
        .collect();

    res.unwrap()
}

fn query_sql(df: DataFrame, sql: &str, table: &str) -> DataFrame {
    let mut context = polars::sql::SQLContext::new();
    context.register(table, df.lazy());
    context
        .execute(sql)
        .expect("Could not execute statement")
        .collect()
        .unwrap()
}
