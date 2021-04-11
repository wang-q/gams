use clap::*;
use garr::*;
use redis::Commands;
use std::process::Command;

use rand::Rng;
use std::collections::{BTreeMap, BTreeSet};

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .about("Test Redis config and connection")
        .after_help(
            r#"
List of actions:

* cli:  find `redis-cli` in $PATH
* test: redis.rs functionality
* info: Command INFO - memory usage of the database
* drop: Command FLUSHDB - drop the database for accepting new data
* dump: Command SAVE - export of the contents of the database

"#,
        )
        .arg(
            Arg::with_name("action")
                .help("What to do")
                .required(true)
                .default_value("test")
                .index(1),
        )
        .arg(
            Arg::with_name("outfile")
                .short("o")
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .empty_values(false)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), std::io::Error> {
    match args.value_of("action").unwrap() {
        "cli" => {
            cli();
        }
        "test" => {
            basics();
            hash();
            list();
            set();
            sorted_set();
            pipe_atomic();
        }
        "info" => {
            info();
        }
        "drop" => {
            drop();
        }
        "dump" => {
            dump();
        }
        _ => unreachable!(),
    };

    Ok(())
}

fn info() {
    let mut conn = connect();
    let info: redis::InfoDict = redis::cmd("INFO")
        .query(&mut conn)
        .expect("Failed to execute INFO");

    let mut output: BTreeMap<&str, String> = BTreeMap::new();
    for key in &[
        "redis_version",
        "os",
        "used_memory_human",
        "total_system_memory_human",
        "maxmemory_human",
        "total_connections_received",
        "total_commands_processed",
    ] {
        output.insert(key, info.get(key).unwrap());
    }

    eprintln!("output = {:#?}", output);
}

fn cli() {
    match Command::new("redis-cli").arg("--version").output() {
        Ok(o) => println!("Find `{:#?}` in $PATH", String::from_utf8(o.stdout)),
        Err(_) => println!("`redis-cli` was not found! Check your $PATH!"),
    }
}

fn drop() {
    let mut conn = connect();
    let output: String = redis::cmd("FLUSHDB")
        .query(&mut conn)
        .expect("Failed to execute FLUSHDB");
    println!("{}", output);
}

fn dump() {
    let mut conn = connect();
    let output: String = redis::cmd("SAVE")
        .query(&mut conn)
        .expect("Failed to execute SAVE");
    println!("{}", output);
}

fn basics() {
    let mut conn = connect();
    println!("******* Running SET, GET, INCR commands *******");

    let _: () = redis::cmd("SET")
        .arg("foo")
        .arg("bar")
        .query(&mut conn)
        .expect("Failed to execute SET for 'foo'");

    let bar: String = redis::cmd("GET")
        .arg("foo")
        .query(&mut conn)
        .expect("Failed to execute GET for 'foo'");
    println!("value for 'foo' = {}", bar);

    //INCR and GET using high-level commands
    let _: () = conn
        .incr("counter", 2)
        .expect("Failed to execute INCR for 'counter'");

    let val: i32 = conn
        .get("counter")
        .expect("Failed to execute GET for 'counter'");

    println!("counter = {}", val);
}

fn hash() {
    let mut conn = connect();

    println!("******* Running HASH commands *******");

    let mut driver: BTreeMap<String, String> = BTreeMap::new();
    let prefix = "redis-driver";

    driver.insert(String::from("name"), String::from("redis-rs"));
    driver.insert(String::from("version"), String::from("0.20.0"));
    driver.insert(
        String::from("repo"),
        String::from("https://github.com/mitsuhiko/redis-rs"),
    );

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", prefix, "rust"))
        .arg(driver)
        .query(&mut conn)
        .expect("Failed to execute HSET");

    let info: BTreeMap<String, String> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", prefix, "rust"))
        .query(&mut conn)
        .expect("Failed to execute HGETALL");

    println!("info for rust redis driver: {:?}", info);

    let _: () = conn
        .hset_multiple(
            format!("{}:{}", prefix, "go"),
            &[
                ("name", "go-redis"),
                ("version", "8.4.6"),
                ("repo", "https://github.com/go-redis/redis"),
            ],
        )
        .expect("Failed to execute HSET");

    let repo_name: String = conn
        .hget(format!("{}:{}", prefix, "go"), "repo")
        .expect("Failed to execute HGET");

    println!("go redis driver repo name: {:?}", repo_name);
}

fn list() {
    let mut conn = connect();
    println!("******* Running LIST commands *******");

    let list_name = "items";

    let _: () = redis::cmd("LPUSH")
        .arg(list_name)
        .arg("item-1")
        .query(&mut conn)
        .expect("Failed to execute LPUSH for 'items'");

    let item: String = conn
        .lpop(list_name)
        .expect("Failed to execute LPOP for 'items'");
    println!("first item: {}", item);

    let _: () = conn.rpush(list_name, "item-2").expect("RPUSH failed");
    let _: () = conn.rpush(list_name, "item-3").expect("RPUSH failed");

    let len: isize = conn
        .llen(list_name)
        .expect("Failed to execute LLEN for 'items'");
    println!("no. of items in list = {}", len);

    let items: Vec<String> = conn
        .lrange(list_name, 0, len - 1)
        .expect("Failed to execute LRANGE for 'items'");
    println!("listing items in list");

    for item in items {
        println!("item: {}", item)
    }
}

fn set() {
    let mut conn = connect();
    println!("******* Running SET commands *******");

    let set_name = "users";

    let _: () = conn
        .sadd(set_name, "user1")
        .expect("Failed to execute SADD for 'users'");
    let _: () = conn
        .sadd(set_name, "user2")
        .expect("Failed to execute SADD for 'users'");

    let ismember: bool = redis::cmd("SISMEMBER")
        .arg(set_name)
        .arg("user1")
        .query(&mut conn)
        .expect("Failed to execute SISMEMBER for 'users'");
    println!("does user1 exist in the set? {}", ismember); //true

    let users: Vec<String> = conn.smembers(set_name).expect("Failed to execute SMEMBERS");
    println!("listing users in set"); //true

    for user in users {
        println!("user: {}", user)
    }
}

fn sorted_set() {
    let mut conn = connect();
    println!("******* Running SORTED SET commands *******");

    let sorted_set = "leaderboard";

    let _: () = redis::cmd("ZADD")
        .arg(sorted_set)
        .arg(rand::thread_rng().gen_range(1..10))
        .arg("player-1")
        .query(&mut conn)
        .expect("Failed to execute ZADD for 'leaderboard'");

    //add many players
    for num in 2..=5 {
        let _: () = conn
            .zadd(
                sorted_set,
                String::from("player-") + &num.to_string(),
                rand::thread_rng().gen_range(1..10),
            )
            .expect("Failed to execute ZADD for 'leaderboard'");
    }

    let count: isize = conn
        .zcard(sorted_set)
        .expect("Failed to execute ZCARD for 'leaderboard'");

    let leaderboard: Vec<(String, isize)> = conn
        .zrange_withscores(sorted_set, 0, count - 1)
        .expect("ZRANGE failed");
    println!("listing players and scores");

    for item in leaderboard {
        println!("{} = {}", item.0, item.1)
    }
}

fn pipe_atomic() {
    let mut conn = connect();
    println!("******* Running MULTI EXEC commands *******");

    redis::pipe()
        .cmd("ZADD")
        .arg("ctg-s:I")
        .arg(1)
        .arg("ctg:I:1")
        .ignore()
        .cmd("ZADD")
        .arg("ctg-s:I")
        .arg(100001)
        .arg("ctg:I:2")
        .ignore()
        .cmd("ZADD")
        .arg("ctg-e:I")
        .arg(100000)
        .arg("ctg:I:1")
        .ignore()
        .cmd("ZADD")
        .arg("ctg-e:I")
        .arg(230218)
        .arg("ctg:I:2")
        .ignore()
        .execute(&mut conn);

    let res_s: BTreeSet<String> = conn.zrangebyscore("ctg-s:I", 0, 1000).unwrap();
    eprintln!("res = {:#?}", res_s);

    let res_e: BTreeSet<String> = conn.zrangebyscore("ctg-e:I", 1100, "+inf").unwrap();
    eprintln!("res = {:#?}", res_e);

    let res: Vec<_> = res_s.intersection(&res_e).collect();
    eprintln!("res = {:#?}", res);

    // MULTI
    // ZRANGESTORE tmp-s:I ctg-s:I 0 1000 BYSCORE
    // ZRANGESTORE tmp-e:I ctg-e:I 1100 +inf BYSCORE
    // ZINTERSTORE tmp-ctg:I 2 tmp-s:I tmp-e:I AGGREGATE MIN
    // DEL tmp-s:I tmp-e:I
    // ZPOPMIN tmp-ctg:I
    // EXEC

    let res: Vec<BTreeMap<String, isize>> = redis::pipe()
        .atomic()
        .cmd("ZRANGESTORE")
        .arg("tmp-s:I")
        .arg("ctg-s:I")
        .arg(0)
        .arg(1000)
        .arg("BYSCORE")
        .ignore()
        .cmd("ZRANGESTORE")
        .arg("tmp-e:I")
        .arg("ctg-e:I")
        .arg(1100)
        .arg("+inf")
        .arg("BYSCORE")
        .ignore()
        .zinterstore_min("tmp-ctg:I", &["tmp-s:I", "tmp-e:I"])
        .ignore()
        .del("tmp-s:I")
        .ignore()
        .del("tmp-e:I")
        .ignore()
        .cmd("ZPOPMIN")
        .arg("tmp-ctg:I")
        .arg(1)
        .query(&mut conn)
        .unwrap();
    let (key, _) = res.first().unwrap().iter().next().unwrap();
    eprintln!("res = {:#?}", res);
    eprintln!("key = {:#?}", key);
}
