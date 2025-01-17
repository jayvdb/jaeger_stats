use clap::Parser;
use jaeger_stats::{set_comma_float, AnomalyParameters, StitchList, StitchParameters, Stitched};
use std::path::Path;

/// Stitching results of different runs of trace_analysis into a single CSV for visualization in Excel
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of files to be stitched
    #[arg(short, long, default_value_t = String::from("input.stitch"))]
    stitch_list: String,

    /// The base of the outputfile
    #[arg(short, long, default_value_t = String::from("stitched"))]
    output: String,

    /// The extension of the outputfile, which can be either 'json' or 'bincode' (bincode is faster, but not guaranteed to be backward portable across rust-versions)
    #[arg(short, long, default_value_t = String::from("json"))]
    ext: String,

    /// generate a csv-output (based on the base as set by the argument output).
    #[arg(short, long, default_value_t = false)]
    csv_output: bool,

    /// Handle input-files that use ',' as floating point separator
    #[arg(long, default_value_t = true)]
    comma_float: bool,

    #[arg(short, long, default_value_t = String::from("anomalies.csv"))]
    anomalies: String,

    #[arg(short, long, default_value_t = 0)]
    drop_count: usize,

    #[arg(long, default_value_t = 0.05)]
    scaled_slope_bound: f64,

    #[arg(long, default_value_t = 5)]
    st_num_points: usize,

    #[arg(long, default_value_t = 0.05)]
    scaled_st_slope_bound: f64,

    #[arg(long, default_value_t = 2.0)]
    l1_dev_bound: f64,
}

fn main() {
    let args = Args::parse();

    let stitch_list_path = Path::new(&args.stitch_list);

    set_comma_float(args.comma_float);

    let stitch_pars = {
        let scaled_slope_bound = args.scaled_slope_bound;
        let st_num_points = args.st_num_points;
        let scaled_st_slope_bound = args.scaled_st_slope_bound;
        let l1_dev_bound = args.l1_dev_bound;
        StitchParameters {
            drop_count: args.drop_count,
            anomaly_pars: AnomalyParameters {
                scaled_slope_bound,
                st_num_points,
                scaled_st_slope_bound,
                l1_dev_bound,
            },
        }
    };

    let stitch_list =
        StitchList::read_stitch_list(stitch_list_path).expect("Failed to read stitchlist-file");
    let stitched = Stitched::build(stitch_list, &stitch_pars);

    let path = Path::new(&args.output);

    if args.csv_output {
        let path = path.with_extension("csv");
        println!("Writing output in CSV-format to: {}", path.display());
        stitched.write_csv(&path);

    }

    {
        let path = path.with_extension(args.ext);
        println!("Writing output in machine-readable format to: {}", path.display());
        stitched.to_json(&path.into_os_string().into_string().unwrap());    
    }

    println!("Stitched output written to: '{}'", path.display());

    let path = Path::new(&args.anomalies);

    println!("Using anomaly parameters: {:?}", stitch_pars.anomaly_pars);
    let num_anomalies = stitched.write_anomalies_csv(path, &stitch_pars.anomaly_pars);
    if num_anomalies > 0 {
        println!(
            "Detected {num_anomalies}.\n\tFor further information check file '{}'",
            path.display()
        );
    } else {
        println!("NO anomalies detected");
    }
}
