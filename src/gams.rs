extern crate clap;
use clap::*;

pub mod cmd_gams;

fn main() -> anyhow::Result<()> {
    let app = Command::new("gams")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with in-Memory Storage")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .subcommand(cmd_gams::env::make_subcommand())
        .subcommand(cmd_gams::status::make_subcommand())
        .subcommand(cmd_gams::gen::make_subcommand())
        .subcommand(cmd_gams::locate::make_subcommand())
        .subcommand(cmd_gams::rg::make_subcommand())
        .subcommand(cmd_gams::clear::make_subcommand())
        .subcommand(cmd_gams::feature::make_subcommand())
        .subcommand(cmd_gams::sw::make_subcommand())
        .subcommand(cmd_gams::anno::make_subcommand())
        .subcommand(cmd_gams::wave::make_subcommand())
        .subcommand(cmd_gams::peak::make_subcommand())
        .subcommand(cmd_gams::tsv::make_subcommand())
        .after_help(
            r###"
"###,
        );

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("env", sub_matches)) => cmd_gams::env::execute(sub_matches),
        Some(("status", sub_matches)) => cmd_gams::status::execute(sub_matches),
        Some(("gen", sub_matches)) => cmd_gams::gen::execute(sub_matches),
        Some(("locate", sub_matches)) => cmd_gams::locate::execute(sub_matches),
        Some(("rg", sub_matches)) => cmd_gams::rg::execute(sub_matches),
        Some(("clear", sub_matches)) => cmd_gams::clear::execute(sub_matches),
        Some(("feature", sub_matches)) => cmd_gams::feature::execute(sub_matches),
        Some(("sw", sub_matches)) => cmd_gams::sw::execute(sub_matches),
        Some(("anno", sub_matches)) => cmd_gams::anno::execute(sub_matches),
        Some(("wave", sub_matches)) => cmd_gams::wave::execute(sub_matches),
        Some(("peak", sub_matches)) => cmd_gams::peak::execute(sub_matches),
        Some(("tsv", sub_matches)) => cmd_gams::tsv::execute(sub_matches),
        _ => unreachable!(),
    }
    .unwrap();

    Ok(())
}

// TODO: sliding windows of waves
// TODO: `gams count`
// TODO: ctgs should be slightly overlapped with each other, 500 bp?
// TODO: `gams swstat` action
// TODO:
//  gams-stat executes sql to generate .tsv
//  rust_xlsxwriter read .tsv, creates charts and writes .xlsx
//  umya-spreadsheet combines them
