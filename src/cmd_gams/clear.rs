use clap::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("clear")
        .about("Clear some parts from Redis")
        .after_help(
            r###"
List of actions:

* feature
    * feature:*
    * cnt:feature:*
* rg
    * rg:*
    * cnt:rg:*
* peak
    * peak:*
    * cnt:peak:*

"###,
        )
        .arg(
            Arg::new("actions")
                .required(true)
                .index(1)
                .num_args(1..)
                .help("What to do"),
        )
        .arg(
            Arg::new("iter")
                .long("iter")
                .action(ArgAction::SetTrue)
                .help("Use a iterator instead of the lua script"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let is_iter = args.get_flag("iter");

    for action in args.get_many::<String>("actions").unwrap() {
        match action.as_str() {
            "feature" => {
                if is_iter {
                    clear_iter("feature:*");
                    clear_iter("cnt:feature:*");
                    clear_iter("bundle:feature:*");
                } else {
                    clear_lua("feature:*");
                    clear_lua("cnt:feature:*");
                    clear_lua("bundle:feature:*");
                }
            }
            "rg" => {
                if is_iter {
                    clear_iter("rg:*");
                    clear_iter("cnt:rg:*");
                    clear_iter("idx:rg:*");
                } else {
                    clear_lua("rg:*");
                    clear_lua("cnt:rg:*");
                    clear_iter("idx:rg:*");
                }
            }
            "peak" => {
                if is_iter {
                    clear_iter("peak:*");
                    clear_iter("cnt:peak:*");
                } else {
                    clear_lua("peak:*");
                    clear_lua("cnt:peak:*");
                }
            }
            _ => unreachable!(),
        };
    }

    Ok(())
}

fn clear_iter(pattern: &str) {
    eprintln!("Clearing pattern {:#?}", pattern);
    // redis connection
    let mut conn = gams::connect();
    let mut conn2 = gams::connect(); // can't use one same `conn` inside an iter

    let iter: redis::Iter<'_, String> = conn.scan_match(pattern).unwrap();
    let mut i: isize = 0;
    for x in iter {
        let _: () = conn2.del(&x).unwrap();
        i += 1;
    }

    eprintln!("    Clear {:#?} keys", i);
}

fn clear_lua(pattern: &str) {
    eprintln!("Clearing pattern {:#?}", pattern);

    let mut conn = gams::connect();

    // https://stackoverflow.com/questions/49055655
    // KEYS is faster than SCAN MATCH
    // I'm already preparing to delete the database, where is the concern for blocking?
    let script = redis::Script::new(
        r###"
local matches = redis.call('KEYS', ARGV[1])

local result = 0
for _,key in ipairs(matches) do
    result = result + redis.call('DEL', key)
end

return result

"###,
    );
    let res: i32 = script.arg(pattern).invoke(&mut conn).unwrap();
    eprintln!("    Clear {:#?} keys", res);
}
