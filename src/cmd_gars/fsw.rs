use clap::*;
use gars::{Ctg, Feature};
use intspan::{IntSpan, Range};
use itertools::Itertools;
use std::collections::HashMap;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("fsw")
        .about("Sliding windows around features")
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
    let parallel = *args.get_one::<usize>("parallel").unwrap();

    //----------------------------
    // Operating
    //----------------------------
    // redis connection
    let mut conn = gars::connect();
    let ctg_of = gars::get_bin_ctg(&mut conn);

    if parallel == 1 {
        let mut writer = intspan::writer(args.get_one::<String>("outfile").unwrap());

        // headers
        let mut headers = vec![
            "range",
            "type",
            "distance",
            "tag",
            "gc_content",
            "gc_mean",
            "gc_stddev",
            "gc_cv",
        ];
        writer.write_all(format!("{}\t{}\n", "ID", headers.join("\t")).as_ref())?;

        // process each contig
        eprintln!("{} contigs to be processed", ctg_of.len());
        for ctg_id in ctg_of.keys().into_iter().sorted() {
            let ctg = ctg_of.get(ctg_id).unwrap();
            let out_string = proc_ctg(ctg, args)?;
            writer.write_all(out_string.as_ref())?;
        }
    }

    Ok(())
}

fn proc_ctg(ctg: &Ctg, args: &ArgMatches) -> anyhow::Result<String> {
    //----------------------------
    // Args
    //----------------------------
    let size = *args.get_one::<i32>("size").unwrap();
    let max = *args.get_one::<i32>("max").unwrap();
    let resize = *args.get_one::<i32>("resize").unwrap();
    let is_range = args.get_flag("range");

    // redis connection
    let mut conn = gars::connect();

    eprintln!(
        "Process {} {}:{}-{}",
        ctg.id, ctg.chr_id, ctg.chr_start, ctg.chr_end
    );

    // local caches of GC-content for each ctg
    let mut cache: HashMap<String, f32> = HashMap::new();

    let parent = IntSpan::from_pair(ctg.chr_start, ctg.chr_end);
    let seq: String = gars::get_ctg_seq(&mut conn, &ctg.id);

    // All features in this ctg
    let features: Vec<Feature> = gars::get_bin_feature(&mut conn, &ctg.id);
    eprintln!("\tThere are {} features", features.len());

    let mut out_string = "".to_string();
    for feature in &features {
        let feature_id = &feature.id;
        let range_start = feature.chr_start;
        let range_end = feature.chr_end;
        let tag = &feature.tag;

        // No need to use Redis counters
        let mut serial: isize = 1;

        let windows = gars::center_sw(&parent, range_start, range_end, size, max);

        for (sw_ints, sw_type, sw_distance) in windows {
            let fsw_id = format!("fsw:{}:{}", feature_id, serial);

            let gc_content = gars::cache_gc_content(
                &Range::from(&ctg.chr_id, sw_ints.min(), sw_ints.max()),
                &parent,
                &seq,
                &mut cache,
            );

            let resized = gars::center_resize(&parent, &sw_ints, resize);
            let re_rg = Range::from(&ctg.chr_id, resized.min(), resized.max());
            let (gc_mean, gc_stddev, gc_cv) =
                gars::cache_gc_stat(&re_rg, &parent, &seq, &mut cache, size, size);

            // prepare to output
            let mut values: Vec<String> = vec![];

            values.push(Range::from(&ctg.chr_id, sw_ints.min(), sw_ints.max()).to_string());
            values.push(sw_type.to_string());
            values.push(format!("{}", sw_distance));
            values.push(tag.to_string());
            values.push(format!("{:.4}", gc_content));
            values.push(format!("{:.4}", gc_mean));
            values.push(format!("{:.4}", gc_stddev));
            values.push(format!("{:.4}", gc_cv));

            serial += 1;

            // outputs
            out_string += &format!("{}\t{}\n", fsw_id, values.join("\t"));
        }
    }

    Ok(out_string)
}
