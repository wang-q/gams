extern crate clap;
use clap::*;

pub mod cmd_gars;

fn main() -> std::io::Result<()> {
    let app = Command::new("gars")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with Rust and rediS")
        .propagate_version(true)
        .arg_required_else_help(true)
        .subcommand(cmd_gars::env::make_subcommand())
        .subcommand(cmd_gars::status::make_subcommand())
        .subcommand(cmd_gars::gen::make_subcommand())
        .subcommand(cmd_gars::locate::make_subcommand())
        .subcommand(cmd_gars::pos::make_subcommand())
        .subcommand(cmd_gars::feature::make_subcommand())
        .subcommand(cmd_gars::sliding::make_subcommand())
        .subcommand(cmd_gars::fsw::make_subcommand())
        .subcommand(cmd_gars::tsv::make_subcommand())
        .subcommand(cmd_gars::wave::make_subcommand());

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("env", sub_matches)) => cmd_gars::env::execute(sub_matches),
        Some(("status", sub_matches)) => cmd_gars::status::execute(sub_matches),
        Some(("gen", sub_matches)) => cmd_gars::gen::execute(sub_matches),
        Some(("locate", sub_matches)) => cmd_gars::locate::execute(sub_matches),
        Some(("pos", sub_matches)) => cmd_gars::pos::execute(sub_matches),
        Some(("feature", sub_matches)) => cmd_gars::feature::execute(sub_matches),
        Some(("fsw", sub_matches)) => cmd_gars::fsw::execute(sub_matches),
        Some(("sliding", sub_matches)) => cmd_gars::sliding::execute(sub_matches),
        Some(("tsv", sub_matches)) => cmd_gars::tsv::execute(sub_matches),
        Some(("wave", sub_matches)) => cmd_gars::wave::execute(sub_matches),
        _ => unreachable!(),
    }
    .unwrap();

    Ok(())
}

// TODO: sliding wave
// TODO: `gars count`
// TODO: `gars clear`
// TODO: pos => range
// TODO: annotations - coding and repeats
