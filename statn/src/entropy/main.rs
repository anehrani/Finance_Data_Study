

use std::env;
use std::io::Read;
use statn::io::read_market_file;
use finance_tools::clean_tails;

mod entropy;
use entropy::{
    calculate_expansion, calculate_jump, calculate_trend, calculate_volatility,
    compute_indicator_stats,
};

/*
Main routine
*/

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: entropy <lookback> <nbins> <version> <filename>");
        eprintln!("  lookback - Lookback for indicators");
        eprintln!("  nbins - Number of bins for entropy calculation");
        eprintln!("  version - 0=raw stat; 1=current-prior; >1=current-longer");
        eprintln!("  filename - name of market file (YYYYMMDD Open High Low Close)");
        std::process::exit(1);
    }

    let lookback: usize = args[1].parse().expect("Invalid lookback");
    let nbins: usize = args[2].parse().expect("Invalid nbins");
    let version: i32 = args[3].parse().expect("Invalid version");
    let filename = &args[4];

    if lookback < 2 {
        eprintln!("Lookback must be at least 2");
        std::process::exit(1);
    }

    let full_lookback = if version == 0 {
        lookback
    } else if version == 1 {
        2 * lookback
    } else if version > 1 {
        (version as usize) * lookback
    } else {
        eprintln!("Version cannot be negative");
        std::process::exit(1);
    };

    // Read market data
    let bars = match read_market_file(filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let nprices = bars.len();
    println!("Market price history read ({} lines)", nprices);
    println!("\nIndicator version {}", version);



    // Trend
    let trend = calculate_trend(&bars, lookback, full_lookback, version);
    compute_indicator_stats(&trend, "Trend", nbins);

    // Volatility
    let volatility = calculate_volatility(&bars, lookback, full_lookback, version);
    compute_indicator_stats(&volatility, "Volatility", nbins);

    // Expansion
    let expansion = calculate_expansion(&bars, lookback, full_lookback, version);
    compute_indicator_stats(&expansion, "Expansion", nbins);

    // Raw jump
    let raw_jump = calculate_jump(&bars, lookback, full_lookback, version);
    compute_indicator_stats(&raw_jump, "RawJump", nbins);

    // Cleaned jump
    let mut cleaned_jump = raw_jump.clone();
    clean_tails(&mut cleaned_jump, 0.05);
    compute_indicator_stats(&cleaned_jump, "CleanedJump", nbins);

    println!("\n\nPress Enter to exit...");
    let _ = std::io::stdin().read(&mut [0u8]);
}