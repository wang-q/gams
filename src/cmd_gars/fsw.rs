use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("fsw")
        .about("Sliding windows around a feature")
        .arg(
            Arg::new("ctg")
                .long("ctg")
                .num_args(1)
                .default_value("ctg:*")
                .help("Sets the full ID or the prefix of ctgs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .num_args(1)
                .default_value("intact")
                .help("Style of sliding windows, intact or center"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .num_args(1)
                .value_parser(value_parser!(i32))
                .default_value("100"),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .num_args(1)
                .value_parser(value_parser!(i32))
                .default_value("20"),
        )
        .arg(
            Arg::new("resize")
                .long("resize")
                .num_args(1)
                .value_parser(value_parser!(i32))
                .default_value("500"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Write a `range` field before the chr_id field"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    // opts
    let size = *args.get_one::<i32>("size").unwrap();
    let max = *args.get_one::<i32>("max").unwrap();
    let resize = *args.get_one::<i32>("resize").unwrap();
    let is_range = args.get_flag("range");

    // redis connection
    let mut conn = connect();

    // headers
    let mut writer = writer(args.get_one::<String>("outfile").unwrap());
    let mut headers = vec![];
    if is_range {
        headers.push("range");
    }
    headers.append(&mut vec![
        "chr_id",
        "chr_start",
        "chr_end",
        "type",
        "distance",
        "tag",
        "gc_content",
        "gc_mean",
        "gc_stddev",
        "gc_cv",
    ]);
    writer.write_all(format!("{}\t{}\n", "ID", headers.join("\t")).as_ref())?;

    // process each contig
    let ctgs: Vec<String> = gars::get_scan_vec(
        &mut conn,
        args.get_one::<String>("ctg").unwrap().to_string(),
    );
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in &ctgs {
        let (chr_id, chr_start, chr_end) = gars::get_key_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

        // local caches of GC-content for each ctg
        let mut cache: HashMap<String, f32> = HashMap::new();

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let seq: String = get_ctg_seq(&mut conn, ctg_id);

        // All features in this ctg
        let features: Vec<String> = get_vec_feature(&mut conn, ctg_id);
        eprintln!("\tThere are {} features", features.len());

        for feature_id in features {
            let (_, range_start, range_end) = gars::get_key_pos(&mut conn, &feature_id);
            let tag: String = conn.hget(&feature_id, "tag").unwrap();

            // No need to use Redis counters
            let mut serial: isize = 1;

            let windows = center_sw(&parent, range_start, range_end, size, max);

            for (sw_intspan, sw_type, sw_distance) in windows {
                let fsw_id = format!("fsw:{}:{}", feature_id, serial);

                let gc_content = cache_gc_content(
                    &Range::from(&chr_id, sw_intspan.min(), sw_intspan.max()),
                    &parent,
                    &seq,
                    &mut cache,
                );

                let resized = center_resize(&parent, &sw_intspan, resize);
                let re_rg = Range::from(&chr_id, resized.min(), resized.max());
                let (gc_mean, gc_stddev, gc_cv) =
                    cache_gc_stat(&re_rg, &parent, &seq, &mut cache, size, size);

                // prepare to output
                let mut values: Vec<String> = vec![];

                if is_range {
                    values
                        .push(Range::from(&chr_id, sw_intspan.min(), sw_intspan.max()).to_string());
                }
                values.push(chr_id.to_string());
                values.push(format!("{}", sw_intspan.min()));
                values.push(format!("{}", sw_intspan.max()));
                values.push(sw_type.to_string());
                values.push(format!("{}", sw_distance));
                values.push(tag.to_string());
                values.push(format!("{:.5}", gc_content));
                values.push(format!("{:.5}", gc_mean));
                values.push(format!("{:.5}", gc_stddev));
                values.push(format!("{:.5}", gc_cv));

                let line = values.join("\t");
                writer.write_all(format!("{}\t{}\n", fsw_id, line).as_ref())?;

                serial += 1;
            }
        }
    }

    Ok(())
}
