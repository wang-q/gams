use clap::*;
use gars::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("clear")
        .about("Clear some parts from Redis")
        .after_help(
            r#"
List of actions:

* feature
    * feature:*
    * cnt:feature:*
* range
    * range:*
    * cnt:range:*
* peak
    * peak:*
    * cnt:peak:*

"#,
        )
        .arg(
            Arg::new("action")
                .help("What to do")
                .required(true)
                .index(1),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match args.value_of("action").unwrap() {
        "feature" => {
            clear("feature:*");
            clear("cnt:feature:*");
        }
        "range" => {
            clear("range:*");
            clear("cnt:range:*");
        }
        "peak" => {
            clear("peak:*");
            clear("cnt:peak:*");
        }
        _ => unreachable!(),
    };

    Ok(())
}

fn clear(pattern: &str) {
    eprintln!("Clearing pattern {:#?}", pattern);
    // redis connection
    let mut conn = connect();
    let mut conn2 = connect(); // can't use one same `conn` inside an iter

    let iter: redis::Iter<'_, String> = conn.scan_match(pattern).unwrap();
    let mut i : isize = 0;
    for x in iter {
        let _: () = conn2.del(&x).unwrap();
        i += 1;
    }

    eprintln!("\tClear {:#?} keys", i);
}
