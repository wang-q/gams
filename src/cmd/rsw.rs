use clap::*;
use garr::*;
use intspan::*;
use redis::Commands;

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("rsw")
        .about("Sliding windows around a range")
        .after_help(
            "\
             --step and --lag should be adjust simultaneously \
             ",
        )
        .arg(
            Arg::with_name("ctg")
                .long("ctg")
                .takes_value(true)
                .default_value("ctg:*")
                .empty_values(false)
                .help("Sets full name or prefix of contigs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::with_name("style")
                .long("style")
                .takes_value(true)
                .default_value("intact")
                .empty_values(false)
                .help("Style of sliding windows, intact or center"),
        )
        .arg(
            Arg::with_name("size")
                .long("size")
                .takes_value(true)
                .default_value("100")
                .empty_values(false),
        )
        .arg(
            Arg::with_name("max")
                .long("max")
                .takes_value(true)
                .default_value("20")
                .empty_values(false),
        )
        .arg(
            Arg::with_name("resize")
                .long("resize")
                .takes_value(true)
                .default_value("500")
                .empty_values(false),
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
    // opts
    let size: i32 = value_t!(args.value_of("size"), i32).unwrap_or_else(|e| {
        eprintln!("Need a integer for --size\n{}", e);
        std::process::exit(1)
    });
    let max: i32 = value_t!(args.value_of("max"), i32).unwrap_or_else(|e| {
        eprintln!("Need a integer for --max\n{}", e);
        std::process::exit(1)
    });
    let resize: i32 = value_t!(args.value_of("resize"), i32).unwrap_or_else(|e| {
        eprintln!("Need a integer for --resize\n{}", e);
        std::process::exit(1)
    });

    // redis connection
    let mut conn = connect();

    // headers
    let mut writer = writer(args.value_of("outfile").unwrap());
    let headers = vec![
        "chr_name",
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
    writer.write_all(format!("{}\t{}\n", "#ID", headers.join("\t")).as_ref())?;

    // process each contig
    let ctgs: Vec<String> =
        garr::get_scan_vec(&mut conn, args.value_of("ctg").unwrap().to_string());
    eprintln!("{} contigs to be processed", ctgs.len());
    for ctg_id in ctgs {
        let (chr_name, chr_start, chr_end) = garr::get_key_pos(&mut conn, &ctg_id);
        eprintln!("Process {} {}:{}-{}", ctg_id, chr_name, chr_start, chr_end);

        let parent = IntSpan::from_pair(chr_start, chr_end);
        let seq: String = conn.get(format!("seq:{}", ctg_id)).unwrap();

        let pattern = format!("range:{}:*", ctg_id);
        let ranges: Vec<String> = garr::get_scan_vec(&mut conn, pattern);
        eprintln!("\t{} ranges to be processed", ranges.len());

        for range_id in ranges {
            // eprintln!("\t{}", range_id);
            let (_, range_start, range_end) = garr::get_key_pos(&mut conn, &range_id);
            let tag: String = conn.hget(&range_id, "tag").unwrap();

            // No need to use Redis counter
            let mut serial: isize = 1;

            let windows = garr::center_sw(&parent, range_start, range_end, size, max);

            for (sw_intspan, sw_type, sw_distance) in windows {
                let rsw_id = format!("rsw:{}:{}", range_id, serial);

                let gc_content = garr::ctg_gc_content(
                    &mut conn,
                    &Range::from(&chr_name, sw_intspan.min(), sw_intspan.max()),
                    &parent,
                    &seq,
                );

                let resized = center_resize(&parent, &sw_intspan, resize);
                let re_rg = Range::from(&chr_name, resized.min(), resized.max());
                let (gc_mean, gc_stddev, gc_cv, gc_snr) =
                    ctg_gc_stat(&mut conn, &re_rg, size, size, &parent, &seq);

                // prepare to output
                let mut values : Vec<String> = vec![];

                values.push(chr_name.to_string());
                push_val_i32(&mut values, sw_intspan.min());
                push_val_i32(&mut values, sw_intspan.max());
                values.push(sw_type.to_string());
                push_val_i32(&mut values, sw_distance);
                values.push(tag.to_string());
                push_val_f32(&mut values, gc_content);
                push_val_f32(&mut values, gc_mean);
                push_val_f32(&mut values, gc_stddev);
                push_val_f32(&mut values, gc_cv);
                push_val_f32(&mut values, gc_snr);

                let line = values.join("\t");
                writer.write_all(format!("{}\t{}\n", rsw_id, line).as_ref())?;

                serial += 1;
            }
        }
    }

    Ok(())
}
