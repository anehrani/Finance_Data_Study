use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const MKTBUF: usize = 2048;

/*
Random number generator - Marsaglia's MWC256
*/

struct RNG {
    q: [u32; 256],
    carry: u32,
    initialized: bool,
    seed: u32,
    i: u8,
}

impl RNG {
    fn new() -> Self {
        RNG {
            q: [0; 256],
            carry: 362436,
            initialized: false,
            seed: 123456789,
            i: 255,
        }
    }

    fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    fn rand32m(&mut self) -> u32 {
        if !self.initialized {
            self.initialized = true;
            let mut j = self.seed;
            for k in 0..256 {
                j = j.wrapping_mul(69069).wrapping_add(12345);
                self.q[k] = j;
            }
        }

        self.i = self.i.wrapping_add(1);
        let t: u64 = (809430660u64)
            .wrapping_mul(self.q[self.i as usize] as u64)
            .wrapping_add(self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    fn unifrand(&mut self) -> f64 {
        self.rand32m() as f64 / 0xFFFFFFFFu32 as f64
    }
}

/*
Compute optimal short-term and long-term lookbacks
for a primitive moving-average crossover system
*/

fn opt_params(
    ncases: usize,
    max_lookback: usize,
    x: &[f64],
) -> (f64, usize, usize, usize, usize) {
    // Returns (total_log_profit, short_term, long_term, nshort, nlong)

    let mut best_perf = -1e60;
    let mut best_short = 1;
    let mut best_long = 2;
    let mut best_nlong = 0;
    let mut best_nshort = 0;

    for ilong in 2..=max_lookback {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut nl = 0;
            let mut ns = 0;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (max_lookback - 1)..(ncases - 1) {
                if i == max_lookback - 1 {
                    // Find the moving averages for the first case
                    short_sum = 0.0;
                    for j in (i - ishort + 1)..=i {
                        short_sum += x[j];
                    }
                    long_sum = short_sum;
                    for j in (i - ilong + 1)..(i - ishort + 1) {
                        long_sum += x[j];
                    }
                } else {
                    // Update the moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let ret = if short_mean > long_mean {
                    // Long position
                    nl += 1;
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    // Short position
                    ns += 1;
                    x[i] - x[i + 1]
                } else {
                    0.0
                };

                total_return += ret;
            }

            if total_return > best_perf {
                best_perf = total_return;
                best_short = ishort;
                best_long = ilong;
                best_nlong = nl;
                best_nshort = ns;
            }
        }
    }

    (best_perf, best_short, best_long, best_nshort, best_nlong)
}

/*
Prepare permutation by computing changes
*/

fn prepare_permute(data: &[f64]) -> Vec<f64> {
    let mut changes = Vec::new();
    for icase in 1..data.len() {
        changes.push(data[icase] - data[icase - 1]);
    }
    changes
}

/*
Perform a permutation shuffle
*/

fn do_permute(rng: &mut RNG, changes: &mut [f64]) {
    let mut i = changes.len();
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = if j >= i { i - 1 } else { j };
        i -= 1;
        changes.swap(i, j);
    }
}

/*
Rebuild prices from changes
*/

fn rebuild_prices(prices: &mut [f64], changes: &[f64], start_idx: usize) {
    for icase in 1..changes.len() + 1 {
        prices[start_idx + icase] = prices[start_idx + icase - 1] + changes[icase - 1];
    }
}

/*
Read market prices from file
*/

fn read_market_file(filename: &str) -> Vec<f64> {
    let mut prices = Vec::new();

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            println!("Reading market file...");

            for (line_num, line) in reader.lines().enumerate() {
                match line {
                    Ok(line_content) => {
                        let trimmed = line_content.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Parse the date (first 8 characters)
                        if trimmed.len() < 9 {
                            eprintln!("Invalid line format at line {}: {}", line_num + 1, trimmed);
                            continue;
                        }

                        let date_str = &trimmed[0..8];
                        if !date_str.chars().all(|c| c.is_ascii_digit()) {
                            eprintln!("Invalid date at line {}: {}", line_num + 1, trimmed);
                            continue;
                        }

                        // Parse the price
                        let price_str = trimmed[9..].trim_start_matches(|c: char| {
                            c == ' ' || c == '\t' || c == ','
                        });

                        match price_str.parse::<f64>() {
                            Ok(price) => {
                                if price > 0.0 {
                                    prices.push(price.ln());
                                }
                            }
                            Err(_) => {
                                eprintln!("Invalid price at line {}: {}", line_num + 1, trimmed);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading line {}: {}", line_num + 1, e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Cannot open market history file {}: {}", filename, e);
            std::process::exit(1);
        }
    }

    prices
}

/*
Main routine
*/

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: mcpt_trn <max_lookback> <nreps> <filename>");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  nreps - Number of MCPT replications (hundreds or thousands)");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        std::process::exit(1);
    }

    let max_lookback: usize = args[1].parse().expect("Invalid max_lookback");
    let nreps: usize = args[2].parse().expect("Invalid nreps");
    let filename = &args[3];

    /*
    Read market prices
    */

    let mut prices = read_market_file(filename);

    if prices.is_empty() {
        eprintln!("No prices read from file");
        std::process::exit(1);
    }

    let nprices = prices.len();
    println!("Market price history read: {} prices", nprices);

    /*
    The market data is read.  Initialize for MCPT.
    For conformity, evaluation period starts at max_lookback-1.
    */

    if nprices - max_lookback < 10 {
        eprintln!("ERROR... Number of prices must be at least 10 greater than max_lookback");
        std::process::exit(1);
    }

    let trend_per_return =
        (prices[nprices - 1] - prices[max_lookback - 1]) / (nprices - max_lookback) as f64;

    // Extract the slice we'll be permuting
    let data_slice = prices[max_lookback - 1..].to_vec();
    let mut changes = prepare_permute(&data_slice);

    let mut rng = RNG::new();

    let mut original = 0.0;
    let mut original_trend_component = 0.0;
    let mut original_nshort = 0;
    let mut original_nlong = 0;
    let mut count = 0;
    let mut mean_training_bias = 0.0;

    /*
    Do MCPT
    */

    for irep in 0..nreps {
        if irep > 0 {
            // Shuffle
            do_permute(&mut rng, &mut changes);
            // Rebuild prices from shuffled changes
            rebuild_prices(&mut prices, &changes, max_lookback - 1);
        }

        let (opt_return, short_lookback, long_lookback, nshort, nlong) =
            opt_params(nprices, max_lookback, &prices);

        let trend_component = (nlong as i32 - nshort as i32) as f64 * trend_per_return;
        let training_bias = opt_return - trend_component;

        println!(
            "{:5}: Ret = {:.3}  Lookback={} {}  NS, NL={} {}  TrndComp={:.4}  TrnBias={:.4}",
            irep, opt_return, short_lookback, long_lookback, nshort, nlong, trend_component, training_bias
        );

        if irep == 0 {
            original = opt_return;
            original_trend_component = trend_component;
            original_nshort = nshort;
            original_nlong = nlong;
            count = 1;
            mean_training_bias = 0.0;
        } else {
            mean_training_bias += training_bias;
            if opt_return >= original {
                count += 1;
            }
        }
    }

    mean_training_bias /= (nreps - 1) as f64;
    let unbiased_return = original - mean_training_bias;
    let skill = unbiased_return - original_trend_component;

    println!(
        "\n\n{} prices were read, {} MCP replications with max lookback = {}",
        nprices, nreps, max_lookback
    );
    println!(
        "\np-value for null hypothesis that system is worthless = {:.4}",
        count as f64 / nreps as f64
    );
    println!(
        "\nTotal trend = {:.4}",
        prices[nprices - 1] - prices[max_lookback - 1]
    );
    println!("\nOriginal nshort = {}", original_nshort);
    println!("\nOriginal nlong = {}", original_nlong);
    println!("\nOriginal return = {:.4}", original);
    println!("\nTrend component = {:.4}", original_trend_component);
    println!("\nTraining bias = {:.4}", mean_training_bias);
    println!("\nSkill = {:.4}", skill);
    println!("\nUnbiased return = {:.4}", unbiased_return);

    println!("\n\nPress Enter to exit...");
    let _ = std::io::stdin().read(&mut [0u8]);
}