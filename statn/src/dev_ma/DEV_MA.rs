/*****************************************************************************/
/*                                                                           */
/*  DEV_MA - Train a thresholded moving-average-crossover system using       */
/*           differential evolution                                          */
/*                                                                           */
/*****************************************************************************/

use std::fs::File;
use std::io::{BufRead, BufReader, Result as IoResult};
use std::cell::RefCell;
use std::f64;

const MKTBUF: usize = 2048; // Alloc for market info in chunks of this many records

thread_local! {
    static LOCAL_STATE: RefCell<LocalState> = RefCell::new(LocalState::new());
}

struct LocalState {
    local_n: usize,
    local_max_lookback: usize,
    local_prices: Vec<f64>,
}

impl LocalState {
    fn new() -> Self {
        LocalState {
            local_n: 0,
            local_max_lookback: 0,
            local_prices: Vec::new(),
        }
    }
}

/*
--------------------------------------------------------------------------------

   Local routine evaluates a thresholded moving-average crossover system
   This computes the total return.  Users may wish to change it to
   compute other criteria.

--------------------------------------------------------------------------------
*/

fn test_system(
    ncases: usize,
    max_lookback: usize,
    x: &[f64],
    long_term: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
    ntrades: &mut usize,
    returns: Option<&mut [f64]>,
) -> f64 {
    let mut short_term = ((0.01 * short_pct * long_term as f64) as usize);
    if short_term < 1 {
        short_term = 1;
    }
    if short_term >= long_term {
        short_term = long_term - 1;
    }

    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    let mut sum = 0.0; // Cumulate performance for this trial
    *ntrades = 0;
    let mut k = 0; // Will index returns

    for i in (max_lookback - 1)..(ncases - 1) {
        // Sum performance across history
        let mut short_mean = 0.0; // Cumulates short-term lookback sum
        for j in (i - short_term + 1)..=i {
            short_mean += x[j];
        }

        let mut long_mean = short_mean; // Cumulates long-term lookback sum
        for j in (i - long_term + 1)..(i - short_term + 1) {
            long_mean += x[j];
        }

        short_mean /= short_term as f64;
        long_mean /= long_term as f64;

        // We now have the short-term and long-term means ending at day i
        // Take our position and cumulate return

        let change = short_mean / long_mean - 1.0; // Fractional difference in MA of log prices

        let ret = if change > long_thresh {
            // Long position
            *ntrades += 1;
            x[i + 1] - x[i]
        } else if change < -short_thresh {
            // Short position
            *ntrades += 1;
            x[i] - x[i + 1]
        } else {
            0.0
        };

        sum += ret;

        if let Some(ret_array) = returns {
            if k < ret_array.len() {
                ret_array[k] = ret;
                k += 1;
            }
        }
    } // For i, summing performance for this trial

    sum
}

/*
--------------------------------------------------------------------------------

   This is the criterion function called from diff_ev.

--------------------------------------------------------------------------------
*/

fn criter(params: &[f64], mintrades: i32) -> f64 {
    LOCAL_STATE.with(|state| {
        let st = state.borrow();

        let long_term = (params[0] + 1.0e-10) as usize;
        let short_pct = params[1];
        let short_thresh = params[2];
        let long_thresh = params[3];

        let mut ntrades = 0;

        let ret_val = test_system(
            st.local_n,
            st.local_max_lookback,
            &st.local_prices,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            &mut ntrades,
            None, // In Rust version, StocBias handling would be done separately
        );

        if (ntrades as i32) >= mintrades {
            ret_val
        } else {
            -1.0e20
        }
    })
}

/*
--------------------------------------------------------------------------------

   Utility function to read market prices from file

--------------------------------------------------------------------------------
*/

fn read_market_prices(filename: &str) -> IoResult<Vec<f64>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Check for empty lines
        if line.len() < 2 {
            continue;
        }

        // Parse and validate date (first 8 characters should be YYYYMMDD)
        if line.len() < 9 {
            eprintln!("Invalid line format: {}", line);
            continue;
        }

        let date_part = &line[..8];
        if !date_part.chars().all(|c| c.is_ascii_digit()) {
            eprintln!("Invalid date in line: {}", line);
            continue;
        }

        // Parse the price (after position 9, skip whitespace and delimiters)
        let price_str = &line[9..];
        let price_str = price_str.trim_start_matches(|c| c == ' ' || c == '\t' || c == ',');

        if let Ok(price) = price_str.parse::<f64>() {
            if price > 0.0 {
                prices.push(price.ln()); // Store log of price
            }
        }
    }

    Ok(prices)
}

/*
--------------------------------------------------------------------------------

   Main routine

--------------------------------------------------------------------------------
*/

fn main() {
    let args: Vec<String> = std::env::args().collect();

    /*
       Process command line parameters
    */

    if args.len() != 4 {
        eprintln!("Usage: DEV_MA  max_lookback  max_thresh  filename");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  max_thresh - Maximum fraction threshold times 10000");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        std::process::exit(1);
    }

    let max_lookback: usize = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid max_lookback value");
            std::process::exit(1);
        }
    };

    let max_thresh: f64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid max_thresh value");
            std::process::exit(1);
        }
    };

    let filename = &args[3];

    /*
       Read market prices
    */

    println!("Reading market file...");

    let prices = match read_market_prices(filename) {
        Ok(p) => {
            println!("Market price history read, {} prices", p.len());
            p
        }
        Err(e) => {
            eprintln!("Cannot open or read market history file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    if prices.is_empty() {
        eprintln!("No prices read from file");
        std::process::exit(1);
    }

    let nprices = prices.len();

    /*
       The market data is read. Set up local state.
    */

    LOCAL_STATE.with(|state| {
        let mut st = state.borrow_mut();
        st.local_n = nprices;
        st.local_max_lookback = max_lookback;
        st.local_prices = prices.clone();
    });

    let low_bounds = [2.0, 0.01, 0.0, 0.0];
    let high_bounds = [max_lookback as f64, 99.0, max_thresh, max_thresh];

    let mintrades = 20;

    /*
       Optimize and print best parameters and performance
    */

    println!("Starting differential evolution optimization...");

    // Note: diff_ev would need to be implemented in Rust
    // For now, we'll create a dummy parameters array
    let mut params = [0.0; 5];

    // Placeholder for diff_ev call
    // In a real implementation, this would call the differential evolution algorithm
    // ret_code = diff_ev(
    //     criter,
    //     4,
    //     1,
    //     100,
    //     10000,
    //     mintrades,
    //     10000000,
    //     300,
    //     0.2,
    //     0.2,
    //     0.3,
    //     &low_bounds,
    //     &high_bounds,
    //     &mut params,
    //     true,
    // );

    println!("\nBest performance = {:.4}", params[4]);
    println!("Variables follow...");
    for i in 0..4 {
        println!("  {:.4}", params[i]);
    }

    /*
       Compute and print parameter sensitivity curves
    */

    // Placeholder for sensitivity analysis
    println!("\nParameter sensitivity analysis would be performed here");

    println!("\nPress Enter to exit...");
    let mut _input = String::new();
    let _ = std::io::stdin().read_line(&mut _input);
}

/*
--------------------------------------------------------------------------------

   Placeholder for diff_ev function signature
   This would need to be implemented based on the Rust version of diff_ev

--------------------------------------------------------------------------------
*/

// #[allow(dead_code)]
// fn diff_ev<F>(
//     criter: F,
//     nvars: usize,
//     nints: usize,
//     popsize: usize,
//     overinit: usize,
//     mintrd: i32,
//     max_evals: usize,
//     max_bad_gen: usize,
//     mutate_dev: f64,
//     pcross: f64,
//     pclimb: f64,
//     low_bounds: &[f64],
//     high_bounds: &[f64],
//     params: &mut [f64],
//     print_progress: bool,
// ) -> i32
// where
//     F: Fn(&[f64], i32) -> f64,
// {
//     // Implementation would go here
//     0
// }

/*
--------------------------------------------------------------------------------

   Placeholder for sensitivity function signature

--------------------------------------------------------------------------------
*/

// #[allow(dead_code)]
// fn sensitivity<F>(
//     criter: F,
//     nvars: usize,
//     nints: usize,
//     num_points: usize,
//     num_trials: usize,
//     mintrades: i32,
//     params: &[f64],
//     low_bounds: &[f64],
//     high_bounds: &[f64],
// ) -> i32
// where
//     F: Fn(&[f64], i32) -> f64,
// {
//     // Implementation would go here
//     0
// }