mod data;
mod analysis;

use matlib::qsortd;
use std::process;

use data::read_market_data;
use indicators::trend::compute_trend;
use indicators::volatility::compute_volatility;
use stats::{find_quantile, find_min_max};
use analysis::{initialize_gap_sizes, gap_analyze, print_gap_analysis};

/*
--------------------------------------------------------------------------------
   Main routine
--------------------------------------------------------------------------------
*/
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        println!("\nUsage:   Lookback  Fractile  Version  Filename");
        println!("  lookback - Lookback for trend and volatility");
        println!("  fractile - Fractile (0-1, typically 0.5) for gap analysis");
        println!("  version - 0=raw stat; 1=current-prior; >1=current-longer");
        println!("  filename - name of market file (YYYYMMDD Price)");
        process::exit(1);
    }

    let lookback = parse_usize(&args[1], "lookback");
    let fractile = parse_f64(&args[2], "fractile");
    let version = parse_usize(&args[3], "version");
    let filename = &args[4];

    if lookback < 2 {
        eprintln!("\n\nLookback must be at least 2");
        process::exit(1);
    }

    let full_lookback = match version {
        0 => lookback,
        1 => 2 * lookback,
        _ => version * lookback,
    };

    // Read market prices
    println!("\nReading market file...");
    let market_data = read_market_data(filename);
    let nprices = market_data.closes.len();
    println!("\nMarket price history read ({} lines)", nprices);
    println!("\n\nIndicator version {}", version);

    if nprices < full_lookback {
        eprintln!("\n\nNot enough data points for the given lookback and version");
        process::exit(1);
    }

    // Initialize for gap analysis
    let gap_size = initialize_gap_sizes();

    // Compute and analyze trend
    let trend = compute_trend(&market_data.closes, lookback, full_lookback, version);
    let (trend_min, trend_max) = find_min_max(&trend);
    let mut trend_sorted = trend.clone();
    qsortd(0, trend.len() - 1, &mut trend_sorted);
    let trend_quantile = find_quantile(&trend_sorted, fractile);

    println!(
        "\n\nTrend  min={:.4}  max={:.4}  {:.3} quantile={:.4}",
        trend_min, trend_max, fractile, trend_quantile
    );

    let gap_count_trend = gap_analyze(&trend, trend_quantile, &gap_size);
    print_gap_analysis(&gap_size, &gap_count_trend, "trend", lookback);

    // Compute and analyze volatility
    let volatility = compute_volatility(&market_data.highs, &market_data.lows, &market_data.closes, lookback, full_lookback, version);
    let (volatility_min, volatility_max) = find_min_max(&volatility);
    let mut volatility_sorted = volatility.clone();
    qsortd(0, volatility.len() - 1, &mut volatility_sorted);
    let volatility_quantile = find_quantile(&volatility_sorted, fractile);

    println!(
        "\n\nVolatility  min={:.4}  max={:.4}  {:.3} quantile={:.4}",
        volatility_min, volatility_max, fractile, volatility_quantile
    );

    let gap_count_volatility = gap_analyze(&volatility, volatility_quantile, &gap_size);
    print_gap_analysis(&gap_size, &gap_count_volatility, "volatility", lookback);

    println!("\n\n Finished...");
}

fn parse_usize(s: &str, param_name: &str) -> usize {
    match s.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid {} parameter", param_name);
            process::exit(1);
        }
    }
}

fn parse_f64(s: &str, param_name: &str) -> f64 {
    match s.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid {} parameter", param_name);
            process::exit(1);
        }
    }
}