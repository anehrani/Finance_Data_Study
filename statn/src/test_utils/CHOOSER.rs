use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const MAX_MARKETS: usize = 1024;
const MAX_NAME_LENGTH: usize = 16;
const BLOCK_SIZE: usize = 4096;
const MAX_CRITERIA: usize = 16;

// Random number generator (Marsaglia MWC256)
struct Rng {
    q: [u32; 256],
    carry: u32,
    i: u8,
    initialized: bool,
    seed: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Rng {
            q: [0; 256],
            carry: 362436,
            i: 255,
            initialized: false,
            seed,
        }
    }

    fn init(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        let mut j = self.seed;
        for k in 0..256 {
            j = j.wrapping_mul(69069).wrapping_add(12345);
            self.q[k] = j;
        }
    }

    fn rand32(&mut self) -> u32 {
        if !self.initialized {
            self.init();
        }

        self.i = self.i.wrapping_add(1);
        let a: u64 = 809430660;
        let t: u64 = a.wrapping_mul(self.q[self.i as usize] as u64).wrapping_add(self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    fn unifrand(&mut self) -> f64 {
        let mult = 1.0 / 0xFFFFFFFF as f64;
        mult * self.rand32() as f64
    }
}

// Performance criteria functions
fn total_return(prices: &[f64]) -> f64 {
    prices[prices.len() - 1] - prices[0]
}

fn sharpe_ratio(prices: &[f64]) -> f64 {
    let n = prices.len() as f64;
    let mean = (prices[prices.len() - 1] - prices[0]) / (n - 1.0);

    let mut var = 1.0e-60;
    for i in 1..prices.len() {
        let diff = (prices[i] - prices[i - 1]) - mean;
        var += diff * diff;
    }

    mean / (var / (n - 1.0)).sqrt()
}

fn profit_factor(prices: &[f64]) -> f64 {
    let mut win_sum = 1.0e-60;
    let mut lose_sum = 1.0e-60;

    for i in 1..prices.len() {
        let ret = prices[i] - prices[i - 1];
        if ret > 0.0 {
            win_sum += ret;
        } else {
            lose_sum -= ret;
        }
    }

    win_sum / lose_sum
}

fn criterion(which: usize, prices: &[f64]) -> f64 {
    match which {
        0 => total_return(prices),
        1 => sharpe_ratio(prices),
        2 => profit_factor(prices),
        _ => -1.0e60,
    }
}

// Permutation routines
fn prepare_permute(nc: usize, nmkt: usize, offset: usize, data: &[Vec<f64>], changes: &mut [Vec<f64>]) {
    for imarket in 0..nmkt {
        for icase in offset..nc {
            changes[imarket][icase] = data[imarket][icase] - data[imarket][icase - 1];
        }
    }
}

fn do_permute(nc: usize, nmkt: usize, offset: usize, data: &mut [Vec<f64>], changes: &mut [Vec<f64>], rng: &mut Rng) {
    let mut i = nc - offset;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = if j >= i { i - 1 } else { j };
        i -= 1;

        for imarket in 0..nmkt {
            let temp = changes[imarket][i + offset];
            changes[imarket][i + offset] = changes[imarket][j + offset];
            changes[imarket][j + offset] = temp;
        }
    }

    for imarket in 0..nmkt {
        for icase in offset..nc {
            data[imarket][icase] = data[imarket][icase - 1] + changes[imarket][icase];
        }
    }
}

struct MarketData {
    names: Vec<String>,
    dates: Vec<Vec<i32>>,
    closes: Vec<Vec<f64>>,
    counts: Vec<usize>,
}

impl MarketData {
    fn new() -> Self {
        MarketData {
            names: Vec::new(),
            dates: Vec::new(),
            closes: Vec::new(),
            counts: Vec::new(),
        }
    }
}

fn parse_csv_field(line: &str, start_pos: &mut usize) -> String {
    let bytes = line.as_bytes();
    
    // Skip delimiters
    while *start_pos < bytes.len() && (bytes[*start_pos] as char == ' ' || bytes[*start_pos] as char == ',' || bytes[*start_pos] as char == '\t') {
        *start_pos += 1;
    }

    // Copy field
    let mut field = String::new();
    while *start_pos < bytes.len() {
        let ch = bytes[*start_pos] as char;
        if ch == ' ' || ch == ',' || ch == '\t' || ch == '/' {
            break;
        }
        field.push(ch);
        *start_pos += 1;
    }

    field
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        eprintln!("\nUSAGE: CHOOSER FileList IS_n OOS1_n nreps");
        eprintln!("  FileList - Text file containing list of competing market history files");
        eprintln!("  IS_n - N of market history records for each selection criterion to analyze");
        eprintln!("  OOS1_n - N of OOS records for choosing best criterion");
        eprintln!("  nreps - Number of Monte-Carlo replications (1 or 0 for none)");
        std::process::exit(0);
    }

    let file_list_name = &args[1];
    let mut is_n: usize = args[2].parse().expect("Invalid IS_n");
    let oos1_n: usize = args[3].parse().expect("Invalid OOS1_n");
    let mut nreps: usize = args[4].parse().expect("Invalid nreps");

    if nreps < 1 {
        nreps = 1;
    }

    if is_n < 2 || oos1_n < 1 {
        eprintln!("\nUSAGE: CHOOSER FileList IS_n OOS1_n nreps");
        eprintln!("  FileList - Text file containing list of competing market history files");
        eprintln!("  IS_n - N of market history records for each selection criterion to analyze");
        eprintln!("  OOS1_n - N of OOS records for choosing best criterion");
        eprintln!("  nreps - Number of Monte-Carlo replications (1 or 0 for none)");
        std::process::exit(0);
    }

    let mut fp_report = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CHOOSER.LOG")?;

    writeln!(fp_report, "CHOOSER log with IS_n={}  OOS1_n={}  Reps={}", is_n, oos1_n, nreps)?;

    // Read market list and data
    let file_list = File::open(file_list_name)?;
    let reader = BufReader::new(file_list);
    let mut market_data = MarketData::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // Extract filename
        let mut filename = String::new();
        for ch in line.chars() {
            if ch.is_alphanumeric() || ch == '_' || ch == '\\' || ch == ':' || ch == '.' {
                filename.push(ch);
            } else {
                break;
            }
        }

        if filename.is_empty() {
            continue;
        }

        // Extract market name from filename
        let mut market_name = String::new();
        if let Some(dot_pos) = filename.rfind('.') {
            let name_part = &filename[..dot_pos];
            if let Some(last_sep) = name_part.rfind(|c| c == '\\' || c == ':' || c == '.') {
                market_name = name_part[last_sep + 1..].to_string();
            } else {
                market_name = name_part.to_string();
            }
        }

        if market_name.len() > MAX_NAME_LENGTH - 1 {
            eprintln!("ERROR... Market name ({}) is too long", market_name);
            std::process::exit(1);
        }

        if market_data.names.len() >= MAX_MARKETS {
            break;
        }

        // Read market file
        if let Ok(file) = File::open(&filename) {
            println!("Reading market file {}...", filename);
            let reader = BufReader::new(file);
            let mut dates = Vec::new();
            let mut closes = Vec::new();
            let mut prior_date = 0;

            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let mut pos = 0;
                    let date_str = parse_csv_field(line, &mut pos);
                    let date: i32 = match date_str.parse() {
                        Ok(d) => d,
                        Err(_) => continue,
                    };

                    let year = date / 10000;
                    let month = (date % 10000) / 100;
                    let day = date % 100;

                    if month < 1 || month > 12 || day < 1 || day > 31 || year < 1800 || year > 2030 {
                        eprintln!("ERROR... Invalid date {} in market file {}", date, filename);
                        std::process::exit(1);
                    }

                    if date <= prior_date {
                        eprintln!("ERROR... Date failed to increase in market file {}", filename);
                        std::process::exit(1);
                    }

                    prior_date = date;

                    let open_str = parse_csv_field(line, &mut pos);
                    let open: f64 = open_str.parse().unwrap_or(0.0);

                    let high_str = parse_csv_field(line, &mut pos);
                    let high: f64 = if high_str.is_empty() { open } else { high_str.parse().unwrap_or(open) };

                    let low_str = parse_csv_field(line, &mut pos);
                    let low: f64 = if low_str.is_empty() { open } else { low_str.parse().unwrap_or(open) };

                    let close_str = parse_csv_field(line, &mut pos);
                    let close: f64 = if close_str.is_empty() { open } else { close_str.parse().unwrap_or(open) };

                    if high < open || high < close || low > open || low > close {
                        eprintln!("ERROR... Open or close outside high/low bounds in market file {}", filename);
                        std::process::exit(1);
                    }

                    dates.push(date);
                    closes.push(close);
                }
            }

            if !dates.is_empty() {
                writeln!(
                    fp_report,
                    "\nMarket file {} had {} records from date {} to {}",
                    filename,
                    dates.len(),
                    dates[0],
                    dates[dates.len() - 1]
                )?;

                market_data.names.push(market_name);
                market_data.dates.push(dates);
                market_data.closes.push(closes);
                market_data.counts.push(dates.len());
            }
        } else {
            eprintln!("ERROR... Cannot open market file {}", filename);
            std::process::exit(1);
        }
    }

    let n_markets = market_data.names.len();

    if n_markets == 0 {
        eprintln!("ERROR... No markets loaded");
        std::process::exit(1);
    }

    // Date alignment: find common dates across all markets
    let mut market_indices = vec![0; n_markets];
    let mut grand_index = 0;
    let mut aligned_dates = Vec::new();
    let mut aligned_closes = vec![Vec::new(); n_markets];

    loop {
        // Find max date at current index
        let mut max_date = 0;
        for i in 0..n_markets {
            if market_indices[i] < market_data.dates[i].len() {
                let date = market_data.dates[i][market_indices[i]];
                if date > max_date {
                    max_date = date;
                }
            }
        }

        if max_date == 0 {
            break;
        }

        // Advance all markets to reach or pass max_date
        let mut all_same_date = true;
        let mut any_exhausted = false;

        for i in 0..n_markets {
            while market_indices[i] < market_data.dates[i].len() {
                let date = market_data.dates[i][market_indices[i]];
                if date >= max_date {
                    break;
                }
                market_indices[i] += 1;
            }

            if market_indices[i] >= market_data.dates[i].len() {
                any_exhausted = true;
                break;
            }

            if market_data.dates[i][market_indices[i]] != max_date {
                all_same_date = false;
            }
        }

        if any_exhausted {
            break;
        }

        // If all markets have the same date, keep it
        if all_same_date {
            aligned_dates.push(max_date);
            for i in 0..n_markets {
                aligned_closes[i].push(market_data.closes[i][market_indices[i]]);
                market_indices[i] += 1;
            }
            grand_index += 1;
        }
    }

    let n_cases = grand_index;

    writeln!(
        fp_report,
        "\n\nMerged database has {} records from date {} to {}",
        n_cases,
        if !aligned_dates.is_empty() { aligned_dates[0] } else { 0 },
        if !aligned_dates.is_empty() { aligned_dates[n_cases - 1] } else { 0 }
    )?;

    // Prepare permutation work arrays if needed
    let mut permute_work = if nreps > 1 {
        let mut pw = vec![vec![0.0; n_cases]; n_markets];
        for i in 0..n_markets {
            pw[i] = aligned_closes[i].clone();
        }
        Some(pw)
    } else {
        None
    };

    // Convert closes to log prices
    for i in 0..n_markets {
        for j in 0..n_cases {
            if aligned_closes[i][j] > 0.0 {
                aligned_closes[i][j] = aligned_closes[i][j].ln();
            }
        }
    }

    // Print OOS2 returns
    let oos2_start = is_n + oos1_n;
    if oos2_start < n_cases {
        writeln!(fp_report, "\n\n25200 * mean return of each market in OOS2 period...")?;
        let mut sum = 0.0;
        for i in 0..n_markets {
            let ret = 25200.0 * (aligned_closes[i][n_cases - 1] - aligned_closes[i][oos2_start - 1]) / (n_cases - oos2_start) as f64;
            sum += ret;
            writeln!(fp_report, "\n{:15} {:9.4}", market_data.names[i], ret)?;
        }
        writeln!(fp_report, "Mean = {:9.4}", sum / n_markets as f64)?;
    }

    // Allocate OOS arrays
    let n_criteria = 3;
    let mut oos1 = vec![vec![0.0; n_cases]; n_criteria];
    let mut oos2 = vec![0.0; n_cases];
    let mut crit_count = vec![0; n_criteria];
    let mut rng = Rng::new(123456789);

    if let Some(pw) = permute_work.as_mut() {
        if is_n > 0 && is_n <= n_cases {
            prepare_permute(is_n, n_markets, 1, &aligned_closes, pw);
        }
        if is_n + oos1_n > is_n && is_n + oos1_n <= n_cases {
            prepare_permute(is_n + oos1_n, n_markets, is_n, &aligned_closes, pw);
        }
        if n_cases > is_n + oos1_n {
            prepare_permute(n_cases, n_markets, is_n + oos1_n, &aligned_closes, pw);
        }
    }

    println!("\n\nComputing");

    let mut crit_pval = vec![1; n_criteria];
    let mut crit_perf = vec![0.0; n_criteria];
    let mut final_pval = 1;
    let mut final_perf = 0.0;

    for irep in 0..nreps {
        if irep > 0 {
            if let Some(pw) = permute_work.as_mut() {
                if is_n > 0 && is_n <= n_cases {
                    do_permute(is_n, n_markets, 1, &mut aligned_closes, pw, &mut rng);
                }
                if is_n + oos1_n > is_n && is_n + oos1_n <= n_cases {
                    do_permute(is_n + oos1_n, n_markets, is_n, &mut aligned_closes, pw, &mut rng);
                }
                if n_cases > is_n + oos1_n {
                    do_permute(n_cases, n_markets, is_n + oos1_n, &mut aligned_closes, pw, &mut rng);
                }
            }
        }

        let mut is_start = 0;
        let mut oos1_start = is_n;
        let mut oos1_end = is_n;
        let mut oos2_end = is_n + oos1_n;

        print!(".");
        std::io::stdout().flush()?;

        loop {
            // Evaluate criteria for all markets
            for icrit in 0..n_criteria {
                let mut best_crit = -1.0e60;
                let mut ibest = 0;

                for imarket in 0..n_markets {
                    if is_start + is_n <= aligned_closes[imarket].len() {
                        let crit_val = criterion(icrit, &aligned_closes[imarket][is_start..is_start + is_n]);
                        if crit_val > best_crit {
                            best_crit = crit_val;
                            ibest = imarket;
                        }
                    }
                }

                if oos1_end > 0 && oos1_end < aligned_closes[ibest].len() {
                    oos1[icrit][oos1_end] = aligned_closes[ibest][oos1_end] - aligned_closes[ibest][oos1_end - 1];
                }
            }

            if oos1_end >= n_cases - 1 {
                break;
            }

            is_start += 1;
            oos1_end += 1;

            if oos1_end - oos1_start < oos1_n {
                continue;
            }

            // Find best criterion in OOS1
            let mut best_crit = -1.0e60;
            let mut ibestcrit = 0;

            for icrit in 0..n_criteria {
                let mut sum = 0.0;
                for i in oos1_start..oos1_end {
                    if i < n_cases {
                        sum += oos1[icrit][i];
                    }
                }
                if sum > best_crit {
                    best_crit = sum;
                    ibestcrit = icrit;
                }
            }

            if irep == 0 {
                crit_count[ibestcrit] += 1;
            }

            // Use best criterion to select market
            let mut best_crit = -1.0e60;
            let mut ibest = 0;

            for imarket in 0..n_markets {
                let window_start = if oos2_end > is_n { oos2_end - is_n } else { 0 };
                if window_start + is_n <= aligned_closes[imarket].len() {
                    let crit_val = criterion(ibestcrit, &aligned_closes[imarket][window_start..window_start + is_n]);
                    if crit_val > best_crit {
                        best_crit = crit_val;
                        ibest = imarket;
                    }
                }
            }

            if oos2_end > 0 && oos2_end < aligned_closes[ibest].len() {
                oos2[oos2_end] = aligned_closes[ibest][oos2_end] - aligned_closes[ibest][oos2_end - 1];
            }

            oos1_start += 1;
            oos2_end += 1;
        }

        // Compute performance metrics
        for i in 0..n_criteria {
            let mut sum = 0.0;
            for j in oos2_start..oos2_end {
                if j < n_cases {
                    sum += oos1[i][j];
                }
            }
            let perf = 25200.0 * sum / (oos2_end - oos2_start).max(1) as f64;

            if irep == 0 {
                crit_perf[i] = perf;
            } else if perf >= crit_perf[i] {
                crit_pval[i] += 1;
            }
        }

        // Compute final performance
        let mut sum = 0.0;
        for i in oos2_start..oos2_end {
            if i < n_cases {
                sum += oos2[i];
            }
        }
        let perf = 25200.0 * sum / (oos2_end - oos2_start).max(1) as f64;

        if irep == 0 {
            final_perf = perf;
        } else if perf >= final_perf {
            final_pval += 1;
        }
    }

    // Print summary
    if nreps > 1 {
        writeln!(fp_report, "\n\n25200 * mean return of each criterion, p-value, and percent of times chosen...")?;
    } else {
        writeln!(fp_report, "\n\n25200 * mean return of each criterion, and percent of times chosen...")?;
    }

    let sum: usize = crit_count.iter().sum();

    for i in 0..n_criteria {
        let name = match i {
            0 => "Total return",
            1 => "Sharpe ratio",
            2 => "Profit factor",
            _ => "ERROR",
        };

        if nreps > 1 {
            writeln!(
                fp_report,
                "\n{:15} {:9.4}  p={:.3}  Chosen {:.1} pct",
                name,
                crit_perf[i],
                crit_pval[i] as f64 / nreps as f64,
                100.0 * crit_count[i] as f64 / sum as f64
            )?;
        } else {
            writeln!(
                fp_report,
                "\n{:15} {:9.4}  Chosen {:.1} pct",
                name,
                crit_perf[i],
                100.0 * crit_count[i] as f64 / sum as f64
            )?;
        }
    }

    if nreps > 1 {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of final system = {:.4}  p={:.3}",
            final_perf,
            final_pval as f64 / nreps as f64
        )?;
    } else {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of final system = {:.4}",
            final_perf
        )?;
    }

    Ok(())
}