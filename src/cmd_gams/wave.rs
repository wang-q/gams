use clap::*;
use std::collections::{HashMap, HashSet};
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("wave")
        .about("GC-wave along a chromosome")
        .after_help(
            r###"
* --step and --lag should be adjust simultaneously
    * step * lag = 1000 for 1000 bp regions

* Crests and troughs are merge separatedly
* Merged peaks would be like "I(+):11551-11740"

* Running in parallel mode will active 1 reader, 1 writer (the main thread)
  and the corresponding number of workers
    * The order of output may differ from serial mode

"###,
        )
        .arg(
            Arg::new("ctg")
                .long("ctg")
                .num_args(1)
                .default_value("ctg:*")
                .help("Sets full name or prefix of contigs, `ctg:I:*` or `ctg:I:2`"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .num_args(1)
                .default_value("100")
                .value_parser(value_parser!(i32))
        )
        .arg(
            Arg::new("step")
                .long("step")
                .num_args(1)
                .default_value("10")
                .value_parser(value_parser!(i32))
        )
        .arg(
            Arg::new("lag")
                .long("lag")
                .num_args(1)
                .default_value("100")
                .value_parser(value_parser!(usize))
                .help("The lag of the moving window"),
        )
        .arg(
            Arg::new("threshold")
                .long("threshold")
                .num_args(1)
                .default_value("3")
                .value_parser(value_parser!(f32))
                .help("The z-score at which the algorithm signals"),
        )
        .arg(
            Arg::new("influence")
                .long("influence")
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(f32))
                .help("The influence (between 0 and 1) of new signals on the mean and standard deviation"),
        )
        .arg(
            Arg::new("signal")
                .long("signal")
                .action(ArgAction::SetTrue)
                .help("Write sliding windows with signals, not just peaks"),
        )
        .arg(
            Arg::new("coverage")
                .long("coverage")
                .short('c')
                .num_args(1)
                .default_value("0.2")
                .value_parser(value_parser!(f32))
                .help("The peaks are merged when the intersection is greater than this value"),
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
    let ctgs: Vec<gams::Ctg> = {
        let mut conn = gams::Conn::new();
        let jsons: Vec<String> = conn.get_scan_values(args.get_one::<String>("ctg").unwrap());
        jsons
            .iter()
            .map(|el| serde_json::from_str(el).unwrap())
            .collect()
    };

    // headers
    writer.write_fmt(format_args!(
        "{}\t{}\t{}\n",
        "#range", "gc_content", "signal"
    ))?;

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
    let opt_step = *args.get_one::<i32>("step").unwrap();
    let opt_lag = *args.get_one::<usize>("lag").unwrap();
    let opt_threshold = *args.get_one::<f32>("threshold").unwrap();
    let opt_influence = *args.get_one::<f32>("influence").unwrap();
    let opt_coverage = *args.get_one::<f32>("coverage").unwrap();
    let is_signal = args.get_flag("signal");

    // redis connection
    let mut conn = gams::Conn::new();

    eprintln!("Process {} {}", ctg.id, ctg.range);

    let parent = intspan::IntSpan::from_pair(ctg.chr_start, ctg.chr_end);
    let windows = gams::sliding(&parent, opt_size, opt_step);

    let ctg_seq: String = conn.get_seq(&ctg.id);

    let mut gcs: Vec<f32> = Vec::with_capacity(windows.len());
    for window in &windows {
        // converted to ctg index
        let from = parent.index(window.min()) as usize;
        let to = parent.index(window.max()) as usize;

        // from <= x < to, zero-based
        let subseq = ctg_seq.get((from - 1)..(to)).unwrap().bytes();
        let gc_content = bio::seq_analysis::gc::gc_content(subseq);
        gcs.push(gc_content);
    }

    let signals = gams::thresholding_algo(&gcs, opt_lag, opt_threshold, opt_influence);

    let mut out_string = "".to_string();
    if is_signal {
        for i in 0..windows.len() {
            out_string += format!(
                "{}:{}\t{}\t{}\n",
                ctg.chr_id,
                windows[i].runlist(),
                gcs[i],
                signals[i],
            )
            .as_str();
        }
    } else {
        let mut merge_of = HashMap::new();
        {
            let crests: Vec<intspan::IntSpan> = windows
                .iter()
                .enumerate()
                .filter(|(i, _)| signals[*i] == 1)
                .map(|(_, el)| el.clone())
                .collect();
            merge_of.extend(merge_ints(crests, opt_coverage));

            let troughs: Vec<intspan::IntSpan> = windows
                .iter()
                .enumerate()
                .filter(|(i, _)| signals[*i] == -1)
                .map(|(_, el)| el.clone())
                .collect();
            merge_of.extend(merge_ints(troughs, opt_coverage));
        }

        // outputs
        let mut seen = HashSet::new();
        for i in 0..windows.len() {
            if signals[i] == 0 {
                continue;
            }

            let runlist = windows[i].runlist();
            if merge_of.contains_key(&runlist) {
                // merged window
                // gc and signal of the first member
                let merge = merge_of.get(&runlist).unwrap();
                if !seen.contains(merge) {
                    out_string +=
                        format!("{}(+):{}\t{}\t{}\n", ctg.chr_id, merge, gcs[i], signals[i],)
                            .as_str();
                    seen.insert(merge.clone());
                }
            } else {
                out_string +=
                    format!("{}:{}\t{}\t{}\n", ctg.chr_id, runlist, gcs[i], signals[i],).as_str();
            }
        }
    }

    out_string
}

fn merge_ints(peaks: Vec<intspan::IntSpan>, opt_coverage: f32) -> HashMap<String, String> {
    // Node weight - &str - node names
    // Edge weight - i32 - append length, back index of strings
    let mut graph = petgraph::prelude::UnGraphMap::new();
    // graph will borrow these runlists
    let runlists: Vec<_> = peaks.iter().map(|el| el.to_string()).collect();
    for i in 0..peaks.len() {
        let ints_i = peaks.get(i).unwrap();
        for j in i + 1..peaks.len() {
            let ints_j = peaks.get(j).unwrap();
            let intersect = ints_i.intersect(ints_j);
            if !intersect.is_empty() {
                let cov_i = ints_i.size() as f32 / intersect.size() as f32;
                let cov_j = ints_j.size() as f32 / intersect.size() as f32;
                if cov_i >= opt_coverage && cov_j >= opt_coverage {
                    graph.add_edge(runlists[i].as_str(), runlists[j].as_str(), ());
                }
            }
        }
    }

    let scc = petgraph::algo::tarjan_scc(&graph);
    let mut merge_of = HashMap::new();
    for cc in &scc {
        let mut merge = intspan::IntSpan::new();
        for member in cc {
            merge.add_runlist(member);
        }

        for member in cc {
            merge_of.insert(member.to_string(), merge.to_string());
        }
    }

    merge_of
}
