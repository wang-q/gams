use clap::*;
use crossbeam::channel::bounded;
use gars::*;
use intspan::*;
use std::io::Write;

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("sliding")
        .about("Sliding windows along a chromosome")
        .after_help(
            r###"
* --step and --lag should be adjust simultaneously

* Running in parallel mode with 1 reader, 1 writer and the corresponding number of workers
    * The order of output may be different from the original

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
                .default_value("50")
                .value_parser(value_parser!(i32))
        )
        .arg(
            Arg::new("lag")
                .long("lag")
                .num_args(1)
                .default_value("1000")
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
    let mut conn = connect();
    let ctgs: Vec<String> = gars::get_scan_vec(
        &mut conn,
        args.get_one::<String>("ctg").unwrap().to_string(),
    );

    if parallel == 1 {
        let mut writer = writer(args.get_one::<String>("outfile").unwrap());

        // headers
        writer.write_fmt(format_args!(
            "{}\t{}\t{}\n",
            "#range", "gc_content", "signal"
        ))?;

        eprintln!("{} contigs to be processed", ctgs.len());
        for ctg_id in ctgs {
            let out_string = proc_ctg(&ctg_id, args)?;
            writer.write_all(out_string.as_ref())?;
        }
    } else {
        proc_ctg_p(&ctgs, args)?;
    }

    Ok(())
}

fn proc_ctg(ctg_id: &String, args: &ArgMatches) -> anyhow::Result<String> {
    //----------------------------
    // Args
    //----------------------------
    let size = *args.get_one::<i32>("size").unwrap();
    let step = *args.get_one::<i32>("step").unwrap();
    let lag = *args.get_one::<usize>("lag").unwrap();
    let threshold = *args.get_one::<f32>("threshold").unwrap();
    let influence = *args.get_one::<f32>("influence").unwrap();

    // redis connection
    let mut conn = connect();

    let (chr_id, chr_start, chr_end) = gars::get_key_pos(&mut conn, &ctg_id);
    eprintln!("Process {} {}:{}-{}", ctg_id, chr_id, chr_start, chr_end);

    let parent = IntSpan::from_pair(chr_start, chr_end);
    let windows = gars::sliding(&parent, size, step);

    let ctg_seq: String = get_ctg_seq(&mut conn, &ctg_id);

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

    let signals = gars::thresholding_algo(&gcs, lag, threshold, influence);

    // outputs
    let mut out_string = "".to_string();
    for i in 0..windows.len() {
        out_string += format!(
            "{}:{}\t{}\t{}\n",
            chr_id,
            windows[i].runlist(),
            gcs[i],
            signals[i],
        )
        .as_str();
    }

    Ok(out_string)
}

// Adopt from https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#create-a-parallel-pipeline
fn proc_ctg_p(ctgs: &Vec<String>, args: &ArgMatches) -> anyhow::Result<()> {
    let parallel = *args.get_one::<usize>("parallel").unwrap();
    let mut writer = writer(args.get_one::<String>("outfile").unwrap());

    // headers
    writer
        .write_fmt(format_args!(
            "{}\t{}\t{}\n",
            "#range", "gc_content", "signal"
        ))
        .unwrap();

    eprintln!("{} contigs to be processed", ctgs.len());

    // Channel 1 - Contigs
    let (snd1, rcv1) = bounded::<String>(10);
    // Channel 2 - Results
    let (snd2, rcv2) = bounded(10);

    crossbeam::scope(|s| {
        //----------------------------
        // Reader thread
        //----------------------------
        s.spawn(|_| {
            for ctg in ctgs {
                snd1.send(ctg.to_string()).unwrap();
            }
            // Close the channel - this is necessary to exit the for-loop in the worker
            drop(snd1);
        });

        //----------------------------
        // Worker threads
        //----------------------------
        for _ in 0..parallel {
            // Send to sink, receive from source
            let (sendr, recvr) = (snd2.clone(), rcv1.clone());
            // Spawn workers in separate threads
            s.spawn(move |_| {
                // Receive until channel closes
                for ctg in recvr.iter() {
                    let out_string = proc_ctg(&ctg, args).unwrap();
                    sendr.send(out_string).unwrap();
                }
            });
        }
        // Close the channel, otherwise sink will never exit the for-loop
        drop(snd2);

        //----------------------------
        // Writer (main) thread
        //----------------------------
        for out_string in rcv2.iter() {
            writer.write_all(out_string.as_ref()).unwrap();
        }
    })
    .unwrap();

    Ok(())
}
