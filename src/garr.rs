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
        .subcommand(cmd::pos::make_subcommand());

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        ("env", Some(sub_matches)) => cmd::env::execute(sub_matches),
        ("status", Some(sub_matches)) => cmd::status::execute(sub_matches),
        ("gen", Some(sub_matches)) => cmd::gen::execute(sub_matches),
        ("pos", Some(sub_matches)) => cmd::pos::execute(sub_matches),
        (_, _) => unreachable!(),
    }?;

    Ok(())
}
