use anyhow::Result;
use clap::Parser;

use chooser::chooser::run_chooser;
use chooser::chooser_dd::run_chooser_dd;

#[derive(Parser, Debug)]
#[command(name = "chooser")]
#[command(about = "Nested walkforward market selection system", long_about = None)]
struct Args {
    /// Mode: "chooser" for Monte Carlo permutation testing, "chooser_dd" for drawdown analysis
    #[arg(value_name = "MODE")]
    mode: String,

    /// Text file containing list of competing market history files
    #[arg(value_name = "FILE_LIST")]
    file_list: String,

    /// Number of market history records for each selection criterion to analyze
    #[arg(value_name = "IS_N")]
    is_n: usize,

    /// Number of OOS records for choosing best criterion
    #[arg(value_name = "OOS1_N")]
    oos1_n: usize,

    /// Number of Monte-Carlo replications (only for chooser mode, 1 or 0 for none)
    #[arg(value_name = "NREPS", default_value = "1")]
    nreps: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.mode.to_lowercase().as_str() {
        "chooser" => {
            println!("Running CHOOSER with Monte Carlo permutation testing...");
            run_chooser(&args.file_list, args.is_n, args.oos1_n, args.nreps)?;
        }
        "chooser_dd" => {
            println!("Running CHOOSER_DD with drawdown analysis...");
            run_chooser_dd(&args.file_list, args.is_n, args.oos1_n)?;
        }
        _ => {
            eprintln!("Error: Invalid mode '{}'. Must be 'chooser' or 'chooser_dd'", args.mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
