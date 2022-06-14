use clap::*;
use gars::*;
use std::collections::HashMap;
use std::fs;
use tera::{Context, Tera};

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("env")
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
        .arg(Arg::new("all").long("all").help("Create all scripts"))
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("gars.env")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // context from ENV variables
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

    // context from args
    let mut opt = HashMap::new();
    opt.insert("outfile", args.get_one::<String>("outfile").unwrap());

    context.insert("opt", &opt);

    // create .env
    eprintln!("Create {}", opt.get("outfile").unwrap());
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/gars.tera.env"))])
        .unwrap();
    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(opt.get("outfile").unwrap(), &vec![rendered.as_str()])?;

    // create scripts
    if args.contains_id("all") {
        // dirs
        fs::create_dir_all("sqls/ddl")?;
        fs::create_dir_all("dumps")?;
        fs::create_dir_all("tsvs")?;

        // files
        gen_peak(&context)?;

        gen_plot_xy(&context)?;

        gen_ddl_ctg(&context)?;
        gen_ddl_fsw(&context)?;

        gen_dql_summary(&context)?;
        gen_dql_fsw(&context)?;

        // TODO: plots
    }

    Ok(())
}

fn gen_peak(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "peak.R";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/peak.tera.R"))])
        .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_plot_xy(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "plot_xy.R";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/plot_xy.tera.R"))])
        .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_ddl_ctg(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "sqls/ddl/ctg.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/ddl/ctg.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_ddl_fsw(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "sqls/ddl/fsw.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/ddl/fsw.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_dql_summary(context: &Context) -> std::result::Result<(), std::io::Error> {
    // summary
    let outname = "sqls/summary.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/summary.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    // summary-type
    let outname = "sqls/summary-type.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/summary-type.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_dql_fsw(context: &Context) -> std::result::Result<(), std::io::Error> {
    // fsw-distance
    let outname = "sqls/fsw-distance.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/fsw-distance.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    // fsw-distance-tag
    let outname = "sqls/fsw-distance-tag.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/fsw-distance-tag.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}
