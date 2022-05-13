use clap::*;
use gars::*;
use intspan::*;
use redis::Commands;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand<'a>() -> Command<'a> {
    Command::new("fsw")
        .about("Sliding windows around a feature")
        .arg(
            Arg::new("ctg")
                .long("ctg")
                .takes_value(true)
                .default_value("ctg:*")
                .forbid_empty_values(true)
                .help("Sets the full ID or the prefix of ctgs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .takes_value(true)
                .default_value("intact")
                .forbid_empty_values(true)
                .help("Style of sliding windows, intact or center"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .takes_value(true)
                .default_value("100")
                .forbid_empty_values(true),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .takes_value(true)
                .default_value("20")
                .forbid_empty_values(true),
        )
        .arg(
            Arg::new("resize")
                .long("resize")
                .takes_value(true)
                .default_value("500")
                .forbid_empty_values(true),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .takes_value(true)
                .default_value("stdout")
                .forbid_empty_values(true)
                .help("Output filename. [stdout] for screen"),
        )
}

// command implementation
pub fn execute(args: &ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // opts
    let size: i32 = args.value_of_t("size").unwrap_or_else(|e| {
        eprintln!("Need a integer for --size\n{}", e);
        std::process::exit(1)
    });
    let max: i32 = args.value_of_t("max").unwrap_or_else(|e| {
        eprintln!("Need a integer for --max\n{}", e);
        std::process::exit(1)
    });
    let resize: i32 = args.value_of_t("resize").unwrap_or_else(|e| {
        eprintln!("Need a integer for --resize\n{}", e);
        std::process::exit(1)
    });

    // redis connection
    let mut conn = connect();

    // headers
    let mut writer = writer(args.value_of("outfile").unwrap());
    let headers = vec![
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
        "gc_snr",
    ];
    writer.write_all(format!("{}\t{}\n", "ID", headers.join("\t")).as_ref())?;

    // process each contig
    let ctgs: Vec<String> =
        gars::get_scan_vec(&mut conn, args.value_of("ctg").unwrap().to_string());
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in &ctgs {
        let (chr_id, chr_start, chr_end) = gars::get_key_pos(&mut conn, ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

        // local caches of GC-content for each ctg
        let mut cache: HashMap<String, f32> = HashMap::new();

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let seq: String = get_ctg_seq(&mut conn, &ctg_id);

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
                let (gc_mean, gc_stddev, gc_cv, gc_snr) =
                    cache_gc_stat(&re_rg, &parent, &seq, &mut cache, size, size);

                // prepare to output
                let mut values: Vec<String> = vec![];

                values.push(format!("{}", chr_id));
                values.push(format!("{}", sw_intspan.min()));
                values.push(format!("{}", sw_intspan.max()));
                values.push(format!("{}", sw_type));
                values.push(format!("{}", sw_distance));
                values.push(format!("{}", tag));
                values.push(format!("{:.5}", gc_content));
                values.push(format!("{:.5}", gc_mean));
                values.push(format!("{:.5}", gc_stddev));
                values.push(format!("{:.5}", gc_cv));
                values.push(format!("{:.5}", gc_snr));

                let line = values.join("\t");
                writer.write_all(format!("{}\t{}\n", fsw_id, line).as_ref())?;

                serial += 1;
            }
        }
    }

    Ok(())
}
