use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const MKTBUF: usize = 2048;

/*
Random number generator - Marsaglia's MWC256
*/

pub struct RNG {
    q: [u32; 256],
    carry: u32,
    initialized: bool,
    seed: u32,
    i: u8,
}

impl RNG {
    pub fn new() -> Self {
        RNG {
            q: [0; 256],
            carry: 362436,
            initialized: false,
            seed: 123456789,
            i: 255,
        }
    }

    pub fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    pub fn rand32m(&mut self) -> u32 {
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

    pub fn unifrand(&mut self) -> f64 {
        self.rand32m() as f64 / 0xFFFFFFFFu32 as f64
    }
}

/*
Bar data structure
*/

#[derive(Clone)]
pub struct BarData {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}

impl BarData {
    pub fn new() -> Self {
        BarData {
            open: Vec::new(),
            high: Vec::new(),
            low: Vec::new(),
            close: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.open.len()
    }

    pub fn push(&mut self, open: f64, high: f64, low: f64, close: f64) {
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
    }

    pub fn validate_ohlc(&self, idx: usize) -> bool {
        self.low[idx] <= self.open[idx]
            && self.low[idx] <= self.close[idx]
            && self.high[idx] >= self.open[idx]
            && self.high[idx] >= self.close[idx]
    }
}

/*
Compute optimal long-term rise and short-term drop
for a primitive mean reversion long-only system
*/

pub fn opt_params(
    ncases: usize,
    lookback: usize,
    bars: &BarData,
) -> (f64, f64, f64, usize) {
    // Returns (total_return, opt_rise, opt_drop, nlong)

    let mut best_perf = -1e60;
    let mut best_rise = 0.005;
    let mut best_drop = 0.0005;
    let mut best_nlong = 0;

    for irise in 1..=50 {
        let rise_thresh = irise as f64 * 0.005;
        for idrop in 1..=50 {
            let drop_thresh = idrop as f64 * 0.0005;

            let mut total_return = 0.0;
            let mut nl = 0;

            for i in lookback..(ncases - 2) {
                let rise = bars.close[i] - bars.close[i - lookback];
                let drop = bars.close[i - 1] - bars.close[i];

                if rise >= rise_thresh && drop >= drop_thresh {
                    let ret = bars.open[i + 2] - bars.open[i + 1];
                    nl += 1;
                    total_return += ret;
                }
            }

            if total_return > best_perf {
                best_perf = total_return;
                best_rise = rise_thresh;
                best_drop = drop_thresh;
                best_nlong = nl;
            }
        }
    }

    (best_perf, best_rise, best_drop, best_nlong)
}

/*
Permutation structure to hold relative price changes
*/

pub struct PermutationData {
    rel_open: Vec<f64>,
    rel_high: Vec<f64>,
    rel_low: Vec<f64>,
    rel_close: Vec<f64>,
}

impl PermutationData {
    pub fn new(nc: usize) -> Self {
        PermutationData {
            rel_open: vec![0.0; nc],
            rel_high: vec![0.0; nc],
            rel_low: vec![0.0; nc],
            rel_close: vec![0.0; nc],
        }
    }
}

/*
Prepare permutation by computing relative changes
*/

pub fn prepare_permute(bars: &BarData, start_idx: usize) -> PermutationData {
    let nc = bars.len() - start_idx;
    let mut perm = PermutationData::new(nc);

    for icase in 1..nc {
        perm.rel_open[icase - 1] = bars.open[start_idx + icase] - bars.close[start_idx + icase - 1];
        perm.rel_high[icase - 1] = bars.high[start_idx + icase] - bars.open[start_idx + icase];
        perm.rel_low[icase - 1] = bars.low[start_idx + icase] - bars.open[start_idx + icase];
        perm.rel_close[icase - 1] = bars.close[start_idx + icase] - bars.open[start_idx + icase];
    }

    perm
}

/*
Perform permutation shuffle with preservation of open-to-open changes
*/

pub fn do_permute(rng: &mut RNG, perm: &mut PermutationData, preserve_oo: bool) {
    let nc = perm.rel_open.len();
    let preserve = if preserve_oo { 1 } else { 0 };

    // Shuffle close-to-open changes
    let mut i = nc - 1 - preserve;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = if j >= i { i - 1 } else { j };
        i -= 1;
        perm.rel_open.swap(i + preserve, j + preserve);
    }

    // Shuffle open-to-close changes
    i = nc - 1 - preserve;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = if j >= i { i - 1 } else { j };
        i -= 1;
        perm.rel_high.swap(i, j);
        perm.rel_low.swap(i, j);
        perm.rel_close.swap(i, j);
    }
}

/*
Rebuild prices from permuted relative changes
*/

pub fn rebuild_prices(bars: &mut BarData, perm: &PermutationData, start_idx: usize) {
    let nc = perm.rel_open.len();

    for icase in 1..nc {
        bars.open[start_idx + icase] = bars.close[start_idx + icase - 1] + perm.rel_open[icase - 1];
        bars.high[start_idx + icase] = bars.open[start_idx + icase] + perm.rel_high[icase - 1];
        bars.low[start_idx + icase] = bars.open[start_idx + icase] + perm.rel_low[icase - 1];
        bars.close[start_idx + icase] = bars.open[start_idx + icase] + perm.rel_close[icase - 1];
    }
}

/*
Read OHLC market data from file
*/

pub fn read_market_file(filename: &str) -> BarData {
    let mut bars = BarData::new();

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

                        // Parse OHLC values
                        let parts: Vec<&str> = trimmed[9..]
                            .split(|c: char| c == ' ' || c == '\t' || c == ',')
                            .filter(|s| !s.is_empty())
                            .collect();

                        if parts.len() < 4 {
                            eprintln!("Insufficient OHLC data at line {}: {}", line_num + 1, trimmed);
                            continue;
                        }

                        let open_price = match parts[0].parse::<f64>() {
                            Ok(p) if p > 0.0 => p.ln(),
                            _ => {
                                eprintln!("Invalid open price at line {}: {}", line_num + 1, parts[0]);
                                continue;
                            }
                        };

                        let high_price = match parts[1].parse::<f64>() {
                            Ok(p) if p > 0.0 => p.ln(),
                            _ => {
                                eprintln!("Invalid high price at line {}: {}", line_num + 1, parts[1]);
                                continue;
                            }
                        };

                        let low_price = match parts[2].parse::<f64>() {
                            Ok(p) if p > 0.0 => p.ln(),
                            _ => {
                                eprintln!("Invalid low price at line {}: {}", line_num + 1, parts[2]);
                                continue;
                            }
                        };

                        let close_price = match parts[3].parse::<f64>() {
                            Ok(p) if p > 0.0 => p.ln(),
                            _ => {
                                eprintln!("Invalid close price at line {}: {}", line_num + 1, parts[3]);
                                continue;
                            }
                        };

                        // Validate OHLC relationships
                        if low_price > open_price
                            || low_price > close_price
                            || high_price < open_price
                            || high_price < close_price
                        {
                            eprintln!(
                                "Invalid OHLC relationships at line {}: O={:.4} H={:.4} L={:.4} C={:.4}",
                                line_num + 1, open_price, high_price, low_price, close_price
                            );
                            continue;
                        }

                        bars.push(open_price, high_price, low_price, close_price);
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

    bars
}

/*
Main routine
*/

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: mcpt_bars <lookback> <nreps> <filename>");
        eprintln!("  lookback - Long-term rise lookback");
        eprintln!("  nreps - Number of MCPT replications (hundreds or thousands)");
        eprintln!("  filename - name of market file (YYYYMMDD Open High Low Close)");
        std::process::exit(1);
    }

    let lookback: usize = args[1].parse().expect("Invalid lookback");
    let nreps: usize = args[2].parse().expect("Invalid nreps");
    let filename = &args[3];

    /*
    Read market prices
    */

    let mut bars = read_market_file(filename);

    if bars.len() == 0 {
        eprintln!("No prices read from file");
        std::process::exit(1);
    }

    let nprices = bars.len();
    println!("Market price history read: {} bars", nprices);

    /*
    The market data is read. Initialize for MCPT.
    Evaluation period starts at lookback.
    */

    if nprices - lookback < 10 {
        eprintln!("ERROR... Number of prices must be at least 10 greater than lookback");
        std::process::exit(1);
    }

    let trend_per_return =
        (bars.open[nprices - 1] - bars.open[lookback + 1]) / (nprices - lookback - 2) as f64;

    let mut perm = prepare_permute(&bars, lookback);
    let mut rng = RNG::new();

    let mut original = 0.0;
    let mut original_trend_component = 0.0;
    let mut original_nlong = 0;
    let mut count = 0;
    let mut mean_training_bias = 0.0;

    /*
    Do MCPT
    */

    for irep in 0..nreps {
        if irep > 0 {
            // Shuffle
            do_permute(&mut rng, &mut perm, true);
            // Rebuild prices from permuted changes
            rebuild_prices(&mut bars, &perm, lookback);
        }

        let (opt_return, opt_rise, opt_drop, nlong) = opt_params(nprices, lookback, &bars);

        let trend_component = nlong as f64 * trend_per_return;
        let training_bias = opt_return - trend_component;

        println!(
            "{:5}: Ret = {:.3}  Rise, drop= {:.4} {:.4}  NL={}  TrndComp={:.4}  TrnBias={:.4}",
            irep, opt_return, opt_rise, opt_drop, nlong, trend_component, training_bias
        );

        if irep == 0 {
            original = opt_return;
            original_trend_component = trend_component;
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
        "\n\n{} prices were read, {} MCP replications with lookback = {}",
        nprices, nreps, lookback
    );
    println!(
        "\n\np-value for null hypothesis that system is worthless = {:.4}",
        count as f64 / nreps as f64
    );
    println!(
        "\nTotal trend = {:.4}",
        bars.open[nprices - 1] - bars.open[lookback + 1]
    );
    println!("\nOriginal nlong = {}", original_nlong);
    println!("\nOriginal return = {:.4}", original);
    println!("\nTrend component = {:.4}", original_trend_component);
    println!("\nTraining bias = {:.4}", mean_training_bias);
    println!("\nSkill = {:.4}", skill);
    println!("\nUnbiased return = {:.4}", unbiased_return);

    println!("\n\nPress Enter to exit...");
    let _ = std::io::stdin().read(&mut [0u8]);
}