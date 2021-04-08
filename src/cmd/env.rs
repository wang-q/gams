use clap::*;
use envy;
use garr::*;
use tera::{Context, Tera};

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("env")
        .about("Create a .env file")
        .after_help(
            r#"
Default values:

* REDIS_HOST - localhost
* REDIS_PORT - 6379
* REDIS_PASSWORD -
* REDIS_TLS - false

"#,
        )
        .arg(
            Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .default_value("garr.env")
                .empty_values(false)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    let mut context = Context::new();

    match envy::from_env::<Config>() {
        Ok(config) => {
            context.insert("host", &config.redis_host);
            context.insert("port", &config.redis_port);
            context.insert("password", &config.redis_password);
            context.insert("tls", &config.redis_tls);
        }
        Err(error) => panic!("{:#?}", error),
    }

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/garr.tera.env"))])
        .unwrap();
    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(args.value_of("outfile").unwrap(), &vec![rendered.as_str()])?;

    Ok(())
}
