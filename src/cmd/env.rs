use clap::*;
use garr::*;
use std::collections::HashMap;
use std::fs;
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
        .arg(Arg::with_name("all").long("all").help("Create all scripts"))
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
    opt.insert("outfile", args.value_of("outfile").unwrap());

    context.insert("opt", &opt);

    // create .env
    eprintln!("Create {}", opt.get("outfile").unwrap());
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/garr.tera.env"))])
        .unwrap();
    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(opt.get("outfile").unwrap(), &vec![rendered.as_str()])?;

    // create scripts
    if args.is_present("all") {
        gen_peak(&context)?;
        gen_plot_xy(&context)?;

        fs::create_dir_all("sqls/ddl")?;
        gen_ddl_ctg(&context)?;
        gen_ddl_rsw(&context)?;

        gen_dql_rsw(&context)?;

        // TODO: worksheet -- summary
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

    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_plot_xy(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "plot_xy.R";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("t", include_str!("../../templates/plot_xy.tera.R"))])
        .unwrap();

    let rendered = tera.render("t", &context).unwrap();
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

    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_ddl_rsw(context: &Context) -> std::result::Result<(), std::io::Error> {
    let outname = "sqls/ddl/rsw.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/ddl/rsw.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}

fn gen_dql_rsw(context: &Context) -> std::result::Result<(), std::io::Error> {
    // rsw-distance
    let outname = "sqls/rsw-distance.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/rsw-distance.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    // rsw-distance-tag
    let outname = "sqls/rsw-distance-tag.sql";
    eprintln!("Create {}", outname);

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "t",
        include_str!("../../templates/dql/rsw-distance-tag.tera.sql"),
    )])
    .unwrap();

    let rendered = tera.render("t", &context).unwrap();
    intspan::write_lines(outname, &vec![rendered.as_str()])?;

    Ok(())
}
