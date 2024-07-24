use clap::*;
use itertools::Itertools;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("sw")
        .about("Sliding windows around features/peaks")
        .arg(
            Arg::new("target")
                .required(false)
                .num_args(1)
                .index(1)
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("feature"),
                    builder::PossibleValue::new("peak"),
                ])
                .default_value("feature")
                .help("Which target"),
        )
        .arg(
            Arg::new("action")
                .required(false)
                .num_args(1)
                .index(2)
                .action(ArgAction::Set)
                .value_parser([
                    builder::PossibleValue::new("gc"),
                    builder::PossibleValue::new("count"),
                ])
                .default_value("gc")
                .help("Which statistics"),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .num_args(1)
                .value_parser([
                    builder::PossibleValue::new("intact"),
                    builder::PossibleValue::new("center"),
                ])
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
            Arg::new("parallel")
                .long("parallel")
                .short('p')
                .value_parser(value_parser!(usize))
                .num_args(1)
                .default_value("1")
                .help("Running in parallel mode, the number of threads"),
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
    //----------------------------
    // Args
    //----------------------------
    let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

    //----------------------------
    // Operating
    //----------------------------
    // redis connection
    let mut conn = gams::Conn::new();
    let ctg_of = conn.get_bundle_ctg(None);
    let mut ctgs = vec![];
    for ctg_id in ctg_of.keys().sorted() {
        ctgs.push(ctg_of.get(ctg_id).unwrap().clone())
    }

    // headers
    let headers = [
        "range",
        "type",
        "distance",
        "tag",
        "gc_content",
        "gc_mean",
        "gc_stddev",
        "gc_cv",
    ];
    writer.write_all(format!("{}\t{}\n", "id", headers.join("\t")).as_ref())?;

    eprintln!("{} contigs to be processed", ctgs.len());

    let rcv = gams::proc_ctg_p(&ctgs, args, proc_ctg);
    for out_string in rcv.iter() {
        writer.write_all(out_string.as_ref())?;
    }

    Ok(())
}

fn proc_ctg(ctg: &gams::Ctg, args: &ArgMatches) -> String {
    //----------------------------
    // Args
    //----------------------------
    let opt_size = *args.get_one::<i32>("size").unwrap();
    let opt_max = *args.get_one::<i32>("max").unwrap();
    let opt_resize = *args.get_one::<i32>("resize").unwrap();

    // redis connection
    let mut conn = gams::Conn::new();

    eprintln!("Process {} {}", ctg.id, ctg.range);

    // local caches of GC-content for each ctg
    let mut cache: HashMap<String, f32> = HashMap::new();

    let parent = intspan::IntSpan::from_pair(ctg.chr_start, ctg.chr_end);
    let seq: String = conn.get_seq(&ctg.id);

    // All features in this ctg
    let jsons: Vec<String> = conn.get_scan_values(&format!("feature:{}:*", ctg.id));
    let features: Vec<gams::Feature> = jsons
        .iter()
        .map(|el| serde_json::from_str(el).unwrap())
        .collect();
    eprintln!("\tThere are {} features", features.len());

    let mut out_string = "".to_string();
    for feature in &features {
        let feature_id = &feature.id;
        let feature_range = intspan::Range::from_str(&feature.range);
        let range_start = feature_range.start;
        let range_end = feature_range.end;
        let tag = &feature.tag;

        // No need to use Redis counters
        let mut serial: isize = 1;

        let windows = gams::center_sw(&parent, range_start, range_end, opt_size, opt_max);

        for (sw_ints, sw_type, sw_distance) in windows {
            let sw_id = format!("sw:{}:{}", feature_id, serial);

            let gc_content = gams::cache_gc_content(
                &intspan::Range::from(&ctg.chr_id, sw_ints.min(), sw_ints.max()),
                &parent,
                &seq,
                &mut cache,
            );

            let resized = gams::center_resize(&parent, &sw_ints, opt_resize);
            let re_rg = intspan::Range::from(&ctg.chr_id, resized.min(), resized.max());
            let (gc_mean, gc_stddev, gc_cv) =
                gams::cache_gc_stat(&re_rg, &parent, &seq, &mut cache, opt_size, opt_size);

            // prepare to output
            let mut values: Vec<String> = vec![];

            values
                .push(intspan::Range::from(&ctg.chr_id, sw_ints.min(), sw_ints.max()).to_string());
            values.push(sw_type.to_string());
            values.push(format!("{}", sw_distance));
            values.push(tag.to_string());
            values.push(format!("{:.4}", gc_content));
            values.push(format!("{:.4}", gc_mean));
            values.push(format!("{:.4}", gc_stddev));
            values.push(format!("{:.4}", gc_cv));

            serial += 1;

            // outputs
            out_string += &format!("{}\t{}\n", sw_id, values.join("\t"));
        }
    }

    out_string
}
