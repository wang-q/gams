use clap::*;
use envy;
use serde::Deserialize;
use tera::{Context, Tera};

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default = "default_redis_host")]
    redis_host: String,
    #[serde(default = "default_redis_port")]
    redis_port: u32,
    redis_password: Option<String>,
    #[serde(default = "default_redis_tls")]
    redis_tls: bool,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u32 {
    6379
}

fn default_redis_tls() -> bool {
    false
}

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("conf")
        .about("Create a config file")
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
                .default_value("garr.conf")
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
    tera.add_raw_templates(vec![("t", include_str!("../../templates/garr.tera.conf"))])
        .unwrap();
    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(args.value_of("outfile").unwrap(), &vec![rendered.as_str()])?;

    Ok(())
}
