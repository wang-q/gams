extern crate clap;
use clap::*;

mod cmd;

fn main() -> std::io::Result<()> {
    let app = App::new("gars")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with Rust and rediS")
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(cmd::env::make_subcommand())
        .subcommand(cmd::status::make_subcommand())
        .subcommand(cmd::gen::make_subcommand())
        .subcommand(cmd::pos::make_subcommand())
        .subcommand(cmd::range::make_subcommand())
        .subcommand(cmd::sliding::make_subcommand())
        .subcommand(cmd::stat::make_subcommand())
        .subcommand(cmd::rsw::make_subcommand())
        .subcommand(cmd::tsv::make_subcommand())
        .subcommand(cmd::wave::make_subcommand());

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        Some(("env", sub_matches)) => cmd::env::execute(sub_matches),
        Some(("status", sub_matches)) => cmd::status::execute(sub_matches),
        Some(("gen", sub_matches)) => cmd::gen::execute(sub_matches),
        Some(("pos", sub_matches)) => cmd::pos::execute(sub_matches),
        Some(("range", sub_matches)) => cmd::range::execute(sub_matches),
        Some(("rsw", sub_matches)) => cmd::rsw::execute(sub_matches),
        Some(("sliding", sub_matches)) => cmd::sliding::execute(sub_matches),
        Some(("stat", sub_matches)) => cmd::stat::execute(sub_matches),
        Some(("tsv", sub_matches)) => cmd::tsv::execute(sub_matches),
        Some(("wave", sub_matches)) => cmd::wave::execute(sub_matches),
        _ => unreachable!(),
    }?;

    Ok(())
}

// TODO: annotations - coding and repeats
