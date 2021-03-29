extern crate clap;
use clap::*;

mod cmd;

fn main() -> std::io::Result<()> {
    let app = App::new("garr")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Genome Analyst with Rust and Redis")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(cmd::status::make_subcommand())
        .subcommand(cmd::env::make_subcommand());

    // Check which subcomamnd the user ran...
    match app.get_matches().subcommand() {
        ("status", Some(sub_matches)) => cmd::status::execute(sub_matches),
        ("env", Some(sub_matches)) => cmd::env::execute(sub_matches),
        (_, _) => unreachable!(),
    }?;

    Ok(())
}
