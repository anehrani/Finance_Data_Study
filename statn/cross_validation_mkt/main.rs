mod criter;
mod cscv_core;
mod get_returns;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

use criter::criter;
use cscv_core::cscvcore;
use get_returns::get_returns;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 4 {
        eprintln!("\nUsage: cross_validation_mkt n_blocks max_lookback filename");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        process::exit(1);
    }
    
    let n_blocks: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error: n_blocks must be a positive integer");
        process::exit(1);
    });
    
    let max_lookback: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: max_lookback must be a positive integer");
        process::exit(1);
    });
    
    let filename = &args[3];
    
    // Read market prices
    println!("\nReading market file...");
    
    let file = File::open(filename).unwrap_or_else(|_| {
        eprintln!("\nCannot open market history file {}", filename);
        process::exit(1);
    });
    
    let reader = BufReader::new(file);
    let mut prices = Vec::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.unwrap_or_else(|_| {
            eprintln!("\nError reading line {} of file {}", line_num + 1, filename);
            process::exit(1);
        });
        
        let line = line.trim();
        if line.len() < 2 {
            break; // Empty line, end of data
        }
        
        // Parse the date (first 8 characters) and do a crude sanity check
        if line.len() < 8 {
            eprintln!("\nInvalid line format at line {} of file {}", line_num + 1, filename);
            process::exit(1);
        }
        
        let date_part = &line[0..8];
        if !date_part.chars().all(|c| c.is_ascii_digit()) {
            eprintln!("\nInvalid date reading line {} of file {}", line_num + 1, filename);
            process::exit(1);
        }
        
        // Parse the price (everything after column 8, skip whitespace and commas)
        let price_part = line[8..].trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        
        let price: f64 = price_part.parse().unwrap_or_else(|_| {
            eprintln!("\nInvalid price reading line {} of file {}", line_num + 1, filename);
            process::exit(1);
        });
        
        if price > 0.0 {
            prices.push(price.ln());
        } else {
            eprintln!("\nInvalid price (must be positive) at line {} of file {}", line_num + 1, filename);
            process::exit(1);
        }
    }
    
    println!("\nMarket price history read");
    
    let nprices = prices.len();
    let n_returns = nprices - max_lookback;
    let n_systems = max_lookback * (max_lookback - 1) / 2;
    
    if nprices < 2 || n_blocks < 2 || max_lookback < 2 || n_returns < n_blocks {
        eprintln!("\nUsage: cross_validation_mkt n_blocks max_lookback filename");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        eprintln!("\nError: Invalid parameters or insufficient data");
        eprintln!("  nprices={}, n_blocks={}, max_lookback={}, n_returns={}", 
                 nprices, n_blocks, max_lookback, n_returns);
        process::exit(1);
    }
    
    println!(
        "\n\nnprices={}  n_blocks={}  max_lookback={}  n_systems={}  n_returns={}",
        nprices, n_blocks, max_lookback, n_systems, n_returns
    );
    
    // Compute returns matrix
    let returns = get_returns(&prices, max_lookback);
    
    // Perform cross-validation
    let prob = cscvcore(n_returns, n_systems, n_blocks, &returns);
    
    // Find return of grand best system
    let mut best_crit = 0.0;
    for i in 0..n_systems {
        let start_idx = i * n_returns;
        let end_idx = start_idx + n_returns;
        let crit = criter(&returns[start_idx..end_idx]);
        if i == 0 || crit > best_crit {
            best_crit = crit;
        }
    }
    
    // Print results
    println!(
        "\n\nnprices={}  n_blocks={}  max_lookback={}  n_systems={}  n_returns={}",
        nprices, n_blocks, max_lookback, n_systems, n_returns
    );
    println!(
        "\n1000 * Grand criterion = {:.4}  Prob = {:.4}",
        1000.0 * best_crit,
        prob
    );
}
