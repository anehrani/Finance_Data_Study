use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const MKTBUF: usize = 2048;

/*
Quicksort for f64 array
*/

fn qsortd(data: &mut [f64]) {
    if data.len() <= 1 {
        return;
    }
    qsort_helper(data, 0, data.len() as i32 - 1);
}

fn qsort_helper(data: &mut [f64], first: i32, last: i32) {
    if first >= last {
        return;
    }

    let split = data[((first + last) / 2) as usize];
    let mut lower = first;
    let mut upper = last;

    loop {
        while split > data[lower as usize] {
            lower += 1;
        }
        while split < data[upper as usize] {
            upper -= 1;
        }

        if lower == upper {
            lower += 1;
            upper -= 1;
        } else if lower < upper {
            data.swap(lower as usize, upper as usize);
            lower += 1;
            upper -= 1;
        }

        if lower > upper {
            break;
        }
    }

    if first < upper {
        qsort_helper(data, first, upper);
    }
    if lower < last {
        qsort_helper(data, lower, last);
    }
}

/*
Clean tails of distribution
*/

fn clean_tails(raw: &mut [f64], tail_frac: f64) {
    let n = raw.len();
    let cover = 1.0 - 2.0 * tail_frac;

    let mut work = raw.to_vec();
    qsortd(&mut work);

    let istart = 0;
    let mut istop = ((cover * (n as f64 + 1.0)) as usize).saturating_sub(1);
    if istop >= n {
        istop = n - 1;
    }

    let mut best = 1e60;
    let mut best_start = 0;
    let mut best_stop = 0;

    let mut i_start = istart;
    let mut i_stop = istop;

    while i_stop < n {
        let range = work[i_stop] - work[i_start];
        if range < best {
            best = range;
            best_start = i_start;
            best_stop = i_stop;
        }
        i_start += 1;
        i_stop += 1;
    }

    let minval = work[best_start];
    let mut maxval = work[best_stop];

    if maxval <= minval {
        maxval *= 1.0 + 1e-10;
        let minval_adj = minval * (1.0 - 1e-10);
        for item in raw.iter_mut() {
            if *item < minval_adj {
                *item = minval_adj;
            }
        }
        return;
    }

    let limit = (maxval - minval) * (1.0 - cover);
    let scale = -1.0 / (maxval - minval);

    for item in raw.iter_mut() {
        if *item < minval {
            *item = minval - limit * (1.0 - (-scale * (minval - *item)).exp());
        } else if *item > maxval {
            *item = maxval + limit * (1.0 - (-scale * (*item - maxval)).exp());
        }
    }
}

/*
Compute linear slope (trend)
*/

fn find_slope(lookback: usize, x_ptr: usize, close: &[f64]) -> f64 {
    let start_idx = if x_ptr >= lookback - 1 {
        x_ptr - lookback + 1
    } else {
        0
    };

    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = i as f64 - 0.5 * (lookback - 1) as f64;
        denom += coef * coef;
        slope += coef * close[start_idx + i];
    }

    slope / denom
}

/*
Compute average true range
*/

fn atr(lookback: usize, high_ptr: usize, low_ptr: usize, close_ptr: usize, high: &[f64], low: &[f64], close: &[f64]) -> f64 {
    let h_start = if high_ptr >= lookback - 1 {
        high_ptr - lookback + 1
    } else {
        0
    };
    let l_start = if low_ptr >= lookback - 1 {
        low_ptr - lookback + 1
    } else {
        0
    };
    let c_start = if close_ptr >= lookback - 1 {
        close_ptr - lookback + 1
    } else {
        0
    };

    let mut sum = 0.0;

    for i in 0..lookback {
        let mut term = high[h_start + i] - low[l_start + i];

        if i > 0 {
            let h_c = high[h_start + i] - close[c_start + i - 1];
            let c_l = close[c_start + i - 1] - low[l_start + i];

            if h_c > term {
                term = h_c;
            }
            if c_l > term {
                term = c_l;
            }
        }
        sum += term;
    }

    sum / lookback as f64
}

/*
Compute range expansion (bad indicator for demo only)
*/

fn range_expansion(lookback: usize, x_ptr: usize, close: &[f64]) -> f64 {
    let start_idx = if x_ptr >= lookback - 1 {
        x_ptr - lookback + 1
    } else {
        0
    };

    let mut recent_high = -1e60;
    let mut recent_low = 1e60;
    let mut older_high = -1e60;
    let mut older_low = 1e60;

    for i in 0..(lookback / 2) {
        if close[start_idx + i] > older_high {
            older_high = close[start_idx + i];
        }
        if close[start_idx + i] < older_low {
            older_low = close[start_idx + i];
        }
    }

    for i in (lookback / 2)..lookback {
        if close[start_idx + i] > recent_high {
            recent_high = close[start_idx + i];
        }
        if close[start_idx + i] < recent_low {
            recent_low = close[start_idx + i];
        }
    }

    (recent_high - recent_low) / (older_high - older_low + 1e-10)
}

/*
Compute price jump
*/

fn jump(lookback: usize, x_ptr: usize, close: &[f64]) -> f64 {
    let start_idx = if x_ptr >= lookback - 1 {
        x_ptr - lookback + 1
    } else {
        0
    };

    let alpha = 2.0 / lookback as f64;
    let mut smoothed = close[start_idx];

    for i in 1..(lookback - 1) {
        smoothed = alpha * close[start_idx + i] + (1.0 - alpha) * smoothed;
    }

    close[start_idx + lookback - 1] - smoothed
}

/*
Compute relative entropy
*/

fn entropy(data: &[f64], nbins: usize) -> f64 {
    let n = data.len();
    if n == 0 || nbins < 2 {
        return 0.0;
    }

    let minval = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let maxval = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let factor = (nbins as f64 - 1e-10) / (maxval - minval + 1e-60);

    let mut count = vec![0; nbins];

    for &x in data {
        let k = ((factor * (x - minval)) as usize).min(nbins - 1);
        count[k] += 1;
    }

    let mut sum = 0.0;
    for &c in &count {
        if c > 0 {
            let p = c as f64 / n as f64;
            sum += p * p.ln();
        }
    }

    -sum / (nbins as f64).ln()
}

/*
Bar data structure
*/

#[derive(Clone)]
struct BarData {
    date: Vec<u32>,
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
}

impl BarData {
    fn new() -> Self {
        BarData {
            date: Vec::new(),
            open: Vec::new(),
            high: Vec::new(),
            low: Vec::new(),
            close: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.open.len()
    }

    fn push(&mut self, date: u32, open: f64, high: f64, low: f64, close: f64) {
        self.date.push(date);
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
    }

    fn validate_ohlc(&self, idx: usize) -> bool {
        self.low[idx] <= self.open[idx]
            && self.low[idx] <= self.close[idx]
            && self.high[idx] >= self.open[idx]
            && self.high[idx] >= self.close[idx]
    }
}

/*
Parse OHLC from line
*/

fn parse_ohlc_line(line: &str) -> Option<(u32, f64, f64, f64, f64)> {
    if line.len() < 9 {
        return None;
    }

    let date_str = &line[0..8];
    if !date_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let full_date: u32 = date_str.parse().ok()?;
    let year = full_date / 10000;
    let month = (full_date / 100) % 100;
    let day = full_date % 100;

    if month < 1 || month > 12 || day < 1 || day > 31 || year < 1800 || year > 2030 {
        return None;
    }

    let parts: Vec<&str> = line[9..]
        .split(|c: char| c == ' ' || c == '\t' || c == ',')
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() < 4 {
        return None;
    }

    let open_price = parts[0].parse::<f64>().ok()?.ln();
    let high_price = parts[1].parse::<f64>().ok()?.ln();
    let low_price = parts[2].parse::<f64>().ok()?.ln();
    let close_price = parts[3].parse::<f64>().ok()?.ln();

    if low_price > open_price
        || low_price > close_price
        || high_price < open_price
        || high_price < close_price
    {
        return None;
    }

    Some((full_date, open_price, high_price, low_price, close_price))
}

/*
Read market file
*/

fn read_market_file(filename: &str) -> Result<BarData, String> {
    let mut bars = BarData::new();
    let mut prior_date = 0u32;

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

                        match parse_ohlc_line(trimmed) {
                            Some((full_date, open, high, low, close)) => {
                                if full_date <= prior_date {
                                    return Err(format!("Date failed to increase at line {}", line_num + 1));
                                }
                                prior_date = full_date;
                                bars.push(full_date, open, high, low, close);
                            }
                            None => {
                                return Err(format!("Invalid data at line {}: {}", line_num + 1, trimmed));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error reading line {}: {}", line_num + 1, e));
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Cannot open file {}: {}", filename, e));
        }
    }

    if bars.len() == 0 {
        return Err("No data read from file".to_string());
    }

    Ok(bars)
}

/*
Compute indicator statistics
*/

fn compute_indicator_stats(
    indicator: &[f64],
    name: &str,
    nbins: usize,
) {
    if indicator.is_empty() {
        return;
    }

    let mut sorted = indicator.to_vec();
    qsortd(&mut sorted);

    let minval = sorted[0];
    let maxval = sorted[sorted.len() - 1];
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        0.5 * (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2])
    };

    let rel_entropy = entropy(indicator, nbins);

    println!(
        "\n{}  min={:.4}  max={:.4}  median={:.4}  relative entropy={:.3}",
        name, minval, maxval, median, rel_entropy
    );
}

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

    let nind = nprices - full_lookback + 1;

    // Trend
    let mut trend = Vec::with_capacity(nind);
    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            find_slope(lookback, k, &bars.close)
        } else if version == 1 {
            find_slope(lookback, k, &bars.close)
                - find_slope(lookback, k.saturating_sub(lookback), &bars.close)
        } else {
            find_slope(lookback, k, &bars.close) - find_slope(full_lookback, k, &bars.close)
        };
        trend.push(val);
    }
    compute_indicator_stats(&trend, "Trend", nbins);

    // Volatility
    let mut volatility = Vec::with_capacity(nind);
    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            atr(lookback, k, k, k, &bars.high, &bars.low, &bars.close)
        } else if version == 1 {
            atr(lookback, k, k, k, &bars.high, &bars.low, &bars.close)
                - atr(
                    lookback,
                    k.saturating_sub(lookback),
                    k.saturating_sub(lookback),
                    k.saturating_sub(lookback),
                    &bars.high,
                    &bars.low,
                    &bars.close,
                )
        } else {
            atr(lookback, k, k, k, &bars.high, &bars.low, &bars.close)
                - atr(full_lookback, k, k, k, &bars.high, &bars.low, &bars.close)
        };
        volatility.push(val);
    }
    compute_indicator_stats(&volatility, "Volatility", nbins);

    // Expansion
    let mut expansion = Vec::with_capacity(nind);
    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            range_expansion(lookback, k, &bars.close)
        } else if version == 1 {
            range_expansion(lookback, k, &bars.close)
                - range_expansion(lookback, k.saturating_sub(lookback), &bars.close)
        } else {
            range_expansion(lookback, k, &bars.close)
                - range_expansion(full_lookback, k, &bars.close)
        };
        expansion.push(val);
    }
    compute_indicator_stats(&expansion, "Expansion", nbins);

    // Raw jump
    let mut raw_jump = Vec::with_capacity(nind);
    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            jump(lookback, k, &bars.close)
        } else if version == 1 {
            jump(lookback, k, &bars.close) - jump(lookback, k.saturating_sub(lookback), &bars.close)
        } else {
            jump(lookback, k, &bars.close) - jump(full_lookback, k, &bars.close)
        };
        raw_jump.push(val);
    }
    compute_indicator_stats(&raw_jump, "RawJump", nbins);

    // Cleaned jump
    let mut cleaned_jump = raw_jump.clone();
    clean_tails(&mut cleaned_jump, 0.05);
    compute_indicator_stats(&cleaned_jump, "CleanedJump", nbins);

    println!("\n\nPress Enter to exit...");
    let _ = std::io::stdin().read(&mut [0u8]);
}