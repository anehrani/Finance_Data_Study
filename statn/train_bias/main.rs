use std::env;
use std::process;

mod rng;
mod opt;
mod test_system;
mod trnbias;
mod selbias;
mod tests;

use opt::OptCriteria;
use trnbias::run_training_bias;
use selbias::run_selection_bias;

enum BiasMode {
    Training,
    Selection,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() != 6 {
        eprintln!("\nUsage: train_bias <mode> <which> <ncases> <trend> <nreps>");
        eprintln!("  mode - 'train' or 'sel' (training bias or selection bias)");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        process::exit(1);
    }

    // Parse mode
    let mode = match args[1].as_str() {
        "train" | "training" => BiasMode::Training,
        "sel" | "selection" => BiasMode::Selection,
        _ => {
            eprintln!("Error: mode must be 'train' or 'sel'");
            process::exit(1);
        }
    };

    let which: u32 = args[2].parse().expect("Error parsing which");

    let which = match OptCriteria::from_u32(which) {
        Some(w) => w,
        None => {
            eprintln!("Error: which must be 0, 1, or 2");
            process::exit(1);
        }
    };

    let ncases: usize = args[3].parse().expect("Error parsing ncases");
    let save_trend: f64 = args[4].parse().expect("Error parsing trend");
    let nreps: usize = args[5].parse().expect("Error parsing nreps");

    // Validate parameters
    if ncases < 2 || nreps < 1 {
        eprintln!("\nUsage: train_bias <mode> <which> <ncases> <trend> <nreps>");
        eprintln!("  mode - 'train' or 'sel' (training bias or selection bias)");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        process::exit(1);
    }

    // Route to appropriate function based on mode
    match mode {
        BiasMode::Training => run_training_bias(which, ncases, save_trend, nreps),
        BiasMode::Selection => run_selection_bias(which, ncases, save_trend, nreps),
    }
}