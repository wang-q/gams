extern crate clap;
use clap::*;

mod cmd;

fn main() -> std::io::Result<()> {
    let app = App::new("garr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with Rust and Redis")
        .setting(AppSettings::ArgRequiredElseHelp)
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
        ("env", Some(sub_matches)) => cmd::env::execute(sub_matches),
        ("status", Some(sub_matches)) => cmd::status::execute(sub_matches),
        ("gen", Some(sub_matches)) => cmd::gen::execute(sub_matches),
        ("pos", Some(sub_matches)) => cmd::pos::execute(sub_matches),
        ("range", Some(sub_matches)) => cmd::range::execute(sub_matches),
        ("rsw", Some(sub_matches)) => cmd::rsw::execute(sub_matches),
        ("sliding", Some(sub_matches)) => cmd::sliding::execute(sub_matches),
        ("stat", Some(sub_matches)) => cmd::stat::execute(sub_matches),
        ("tsv", Some(sub_matches)) => cmd::tsv::execute(sub_matches),
        ("wave", Some(sub_matches)) => cmd::wave::execute(sub_matches),
        (_, _) => unreachable!(),
    }?;

    Ok(())
}

// TODO: annotations - coding and repeats
