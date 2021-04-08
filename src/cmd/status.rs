use clap::*;
use garr::*;
use redis::Commands;

use rand::Rng;
use std::collections::BTreeMap;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .about("Test Redis config and connection")
        .arg(
            Arg::with_name("action")
                .help("What to do: test, or info")
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
        "test" => {
            basics();
            hash();
            list();
            set();
            sorted_set();
        }
        "info" => {
            info();
        }
        _ => unreachable!(),
    };

    Ok(())
}

fn info() {
    let mut conn = connect();
    let info: redis::InfoDict = redis::cmd("INFO")
        .query(&mut conn)
        .expect("failed to execute INFO");

    let mut output: BTreeMap<&str, String> = BTreeMap::new();
    for key in vec![
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

fn basics() {
    let mut conn = connect();
    println!("******* Running SET, GET, INCR commands *******");

    let _: () = redis::cmd("SET")
        .arg("foo")
        .arg("bar")
        .query(&mut conn)
        .expect("failed to execute SET for 'foo'");

    let bar: String = redis::cmd("GET")
        .arg("foo")
        .query(&mut conn)
        .expect("failed to execute GET for 'foo'");
    println!("value for 'foo' = {}", bar);

    //INCR and GET using high-level commands
    let _: () = conn
        .incr("counter", 2)
        .expect("failed to execute INCR for 'counter'");

    let val: i32 = conn
        .get("counter")
        .expect("failed to execute GET for 'counter'");

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
        .expect("failed to execute HSET");

    let info: BTreeMap<String, String> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", prefix, "rust"))
        .query(&mut conn)
        .expect("failed to execute HGETALL");

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
        .expect("failed to execute HSET");

    let repo_name: String = conn
        .hget(format!("{}:{}", prefix, "go"), "repo")
        .expect("failed to execute HGET");

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
        .expect("failed to execute LPUSH for 'items'");

    let item: String = conn
        .lpop(list_name)
        .expect("failed to execute LPOP for 'items'");
    println!("first item: {}", item);

    let _: () = conn.rpush(list_name, "item-2").expect("RPUSH failed");
    let _: () = conn.rpush(list_name, "item-3").expect("RPUSH failed");

    let len: isize = conn
        .llen(list_name)
        .expect("failed to execute LLEN for 'items'");
    println!("no. of items in list = {}", len);

    let items: Vec<String> = conn
        .lrange(list_name, 0, len - 1)
        .expect("failed to execute LRANGE for 'items'");
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
        .expect("failed to execute SADD for 'users'");
    let _: () = conn
        .sadd(set_name, "user2")
        .expect("failed to execute SADD for 'users'");

    let ismember: bool = redis::cmd("SISMEMBER")
        .arg(set_name)
        .arg("user1")
        .query(&mut conn)
        .expect("failed to execute SISMEMBER for 'users'");
    println!("does user1 exist in the set? {}", ismember); //true

    let users: Vec<String> = conn.smembers(set_name).expect("failed to execute SMEMBERS");
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
        .expect("failed to execute ZADD for 'leaderboard'");

    //add many players
    for num in 2..=5 {
        let _: () = conn
            .zadd(
                sorted_set,
                String::from("player-") + &num.to_string(),
                rand::thread_rng().gen_range(1..10),
            )
            .expect("failed to execute ZADD for 'leaderboard'");
    }

    let count: isize = conn
        .zcard(sorted_set)
        .expect("failed to execute ZCARD for 'leaderboard'");

    let leaderboard: Vec<(String, isize)> = conn
        .zrange_withscores(sorted_set, 0, count - 1)
        .expect("ZRANGE failed");
    println!("listing players and scores");

    for item in leaderboard {
        println!("{} = {}", item.0, item.1)
    }
}
