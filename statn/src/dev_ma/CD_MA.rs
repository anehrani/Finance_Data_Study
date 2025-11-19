use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const MKTBUF: usize = 2048;

extern "C" {
    fn cv_train(
        n: i32,
        nvars: i32,
        nfolds: i32,
        xx: *mut f64,
        yy: *mut f64,
        ww: *mut f64,
        lambdas: *mut f64,
        lambda_oos: *mut f64,
        work: *mut f64,
        covar_updates: i32,
        n_lambda: i32,
        alpha: f64,
        maxits: i32,
        eps: f64,
        fast_test: i32,
    ) -> f64;
}

struct CoordinateDescent {
    nvars: i32,
    ncases: i32,
    covar_updates: i32,
    n_lambda: i32,
    ok: bool,
    beta: Vec<f64>,
    explained: f64,
    xmeans: Vec<f64>,
    xscales: Vec<f64>,
    ymean: f64,
    yscale: f64,
    lambda_beta: Vec<f64>,
    lambdas: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
    w: Vec<f64>,
    resid: Vec<f64>,
    xinner: Vec<f64>,
    yinner: Vec<f64>,
    xssvec: Vec<f64>,
}

impl CoordinateDescent {
    fn new(nvars: i32, ncases: i32, _weighted: i32, covar_updates: i32, n_lambda: i32) -> Self {
        let nvars_usize = nvars as usize;
        let ncases_usize = ncases as usize;

        CoordinateDescent {
            nvars,
            ncases,
            covar_updates,
            n_lambda,
            ok: true,
            beta: vec![0.0; nvars_usize],
            explained: 0.0,
            xmeans: vec![0.0; nvars_usize],
            xscales: vec![0.0; nvars_usize],
            ymean: 0.0,
            yscale: 0.0,
            lambda_beta: vec![0.0; (n_lambda as usize) * nvars_usize],
            lambdas: vec![0.0; n_lambda as usize],
            x: vec![0.0; ncases_usize * nvars_usize],
            y: vec![0.0; ncases_usize],
            w: vec![],
            resid: vec![0.0; ncases_usize],
            xinner: vec![0.0; nvars_usize * nvars_usize],
            yinner: vec![0.0; nvars_usize],
            xssvec: vec![0.0; nvars_usize],
        }
    }

    fn get_data(&mut self, istart: i32, n: i32, x: &[f64], y: &[f64], w: Option<&[f64]>) {
        let start = istart as usize;
        let count = n as usize;
        let nvars_usize = self.nvars as usize;

        for i in 0..count {
            for j in 0..nvars_usize {
                self.x[i * nvars_usize + j] = x[start * nvars_usize + i * nvars_usize + j];
            }
            self.y[i] = y[start + i];
        }

        if let Some(weights) = w {
            self.w = weights[start..start + count].to_vec();
        }
    }

    fn core_train(&mut self, alpha: f64, lambda: f64, maxits: i32, eps: f64, fast_test: i32, _warm_start: i32) {
        unsafe {
            cv_train(
                self.ncases,
                self.nvars,
                10,
                self.x.as_mut_ptr(),
                self.y.as_mut_ptr(),
                if self.w.is_empty() { std::ptr::null_mut() } else { self.w.as_mut_ptr() },
                self.lambdas.as_mut_ptr(),
                self.lambda_beta.as_mut_ptr(),
                self.resid.as_mut_ptr(),
                self.covar_updates,
                self.n_lambda,
                alpha,
                maxits,
                eps,
                fast_test,
            );
        }
    }
}

fn indicators(nind: usize, x: &[f64], short_term: usize, long_term: usize, inds: &mut [f64]) {
    let mut xptr_offset = if long_term > x.len() { 0 } else { x.len() - long_term };

    for i in 0..nind {
        let k = i + long_term - 1;
        let mut short_mean = 0.0;

        for j in (k - short_term + 1)..=k {
            if xptr_offset + j < x.len() {
                short_mean += x[xptr_offset + j];
            }
        }

        let mut long_mean = short_mean;
        for j in (k - long_term + 1)..(k - short_term + 1) {
            if xptr_offset + j < x.len() {
                long_mean += x[xptr_offset + j];
            }
        }

        short_mean /= short_term as f64;
        long_mean /= long_term as f64;
        inds[i] = short_mean - long_mean;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 6 {
        eprintln!("\nUsage: CD_MA  lookback_inc  n_long  n_short  alpha  filename");
        eprintln!("  lookback_inc - increment to long-term lookback");
        eprintln!("  n_long - Number of long-term lookbacks");
        eprintln!("  n_short - Number of short-term lookbacks");
        eprintln!("  alpha - Alpha, (0-1]");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        std::process::exit(1);
    }

    let lookback_inc: i32 = args[1].parse().expect("Invalid lookback_inc");
    let n_long: i32 = args[2].parse().expect("Invalid n_long");
    let n_short: i32 = args[3].parse().expect("Invalid n_short");
    let mut alpha: f64 = args[4].parse().expect("Invalid alpha");
    let filename = &args[5];

    if alpha >= 1.0 {
        eprintln!("Alpha must be less than 1.");
        std::process::exit(1);
    }

    let mut fp_results = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CD_MA.LOG")?;

    writeln!(fp_results, "Starting CD_MA with alpha = {:.4}", alpha)?;

    // Read market prices
    let mut prices = Vec::new();

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            println!("\nReading market file...");

            for line in reader.lines() {
                if let Ok(line) = line {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.len() < 10 {
                        continue;
                    }

                    // Parse date (crude sanity check)
                    if !trimmed[0..8].chars().all(|c| c.is_ascii_digit()) {
                        eprintln!("\nInvalid date in line: {}", line);
                        std::process::exit(1);
                    }

                    // Parse price
                    let price_str = &trimmed[9..];
                    let price_str = price_str.trim_start_matches(|c: char| c.is_whitespace() || c == ',');

                    if let Ok(mut price) = price_str.parse::<f64>() {
                        if price > 0.0 {
                            price = price.ln();
                        }
                        prices.push(price);
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("\n\nCannot open market history file {}", filename);
            std::process::exit(1);
        }
    }

    let nprices = prices.len();
    println!("\nMarket price history read");

    // Initialize computation
    let n_lambdas = 50;
    let nvars = (n_long * n_short) as usize;
    let n_test = 252;
    let max_lookback = (n_long * lookback_inc) as usize;
    let n_train = nprices - n_test - max_lookback;

    if n_train < (n_long * n_short) as usize + 10 {
        eprintln!("\nERROR... Too little training data for parameters.");
        std::process::exit(1);
    }

    let k = if n_train > n_test { n_train } else { n_test };

    let mut inds = vec![0.0; k];
    let mut targets = vec![0.0; k];
    let mut data = vec![0.0; k * nvars];
    let mut lambdas = vec![0.0; n_lambdas];
    let mut lambda_oos = vec![0.0; n_lambdas];
    let mut work = vec![0.0; n_train];

    // Compute and save indicators for training set
    let mut var_idx = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = std::cmp::max(1, long_lookback * (ishort + 1) / (n_short + 1));
            let start_idx = max_lookback.saturating_sub(1);
            if start_idx < prices.len() {
                indicators(
                    n_train,
                    &prices[start_idx..],
                    short_lookback as usize,
                    long_lookback as usize,
                    &mut inds[0..n_train],
                );
                for i in 0..n_train {
                    data[i * nvars + var_idx as usize] = inds[i];
                }
            }
            var_idx += 1;
        }
    }

    // Compute and save targets for training set
    let pptr_start = max_lookback.saturating_sub(1);
    for i in 0..n_train {
        if pptr_start + i + 1 < prices.len() {
            targets[i] = prices[pptr_start + i + 1] - prices[pptr_start + i];
        }
    }

    // Compute and print optimal lambda
    let lambda = if alpha <= 0.0 {
        alpha = 0.5;
        writeln!(fp_results, "\n\nUser specified negative alpha, so lambda = 0")?;
        0.0
    } else {
        let lambda = unsafe {
            cv_train(
                n_train as i32,
                nvars as i32,
                10,
                data.as_mut_ptr(),
                targets.as_mut_ptr(),
                std::ptr::null_mut(),
                lambdas.as_mut_ptr(),
                lambda_oos.as_mut_ptr(),
                work.as_mut_ptr(),
                1,
                n_lambdas as i32,
                alpha,
                1000,
                1.0e-9,
                1,
            )
        };
        writeln!(
            fp_results,
            "\n\nCross validation gave optimal lambda = {:.4}  XVAL computation below...",
            lambda
        )?;
        writeln!(fp_results, "\n  Lambda   OOS explained")?;
        for i in 0..n_lambdas {
            writeln!(fp_results, "\n{:8.4} {:12.4}", lambdas[i], lambda_oos[i])?;
        }
        lambda
    };

    // Train the model and print beta coefficients
    let mut cd = CoordinateDescent::new(nvars as i32, n_train as i32, 0, 1, 0);
    cd.get_data(0, n_train as i32, &data, &targets, None);
    cd.core_train(alpha, lambda, 1000, 1.0e-7, 1, 0);

    writeln!(
        fp_results,
        "\n\nBetas, with in-sample explained variance = {:.5} percent",
        100.0 * cd.explained
    )?;
    writeln!(
        fp_results,
        "\nRow label is long-term lookback; Columns run from smallest to largest short-term lookback"
    )?;

    var_idx = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        write!(fp_results, "\n{:5} ", long_lookback)?;
        for _ishort in 0..n_short {
            if cd.beta[var_idx as usize] != 0.0 {
                write!(fp_results, "{:9.4}", cd.beta[var_idx as usize])?;
            } else {
                write!(fp_results, "    ---- ")?;
            }
            var_idx += 1;
        }
    }

    // Compute and save indicators for test set
    var_idx = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = std::cmp::max(1, long_lookback * (ishort + 1) / (n_short + 1));
            let start_idx = n_train + max_lookback.saturating_sub(1);
            if start_idx < prices.len() {
                indicators(
                    n_test,
                    &prices[start_idx..],
                    short_lookback as usize,
                    long_lookback as usize,
                    &mut inds[0..n_test],
                );
                for i in 0..n_test {
                    data[i * nvars + var_idx as usize] = inds[i];
                }
            }
            var_idx += 1;
        }
    }

    // Compute and save targets for test set
    let pptr_start = n_train + max_lookback.saturating_sub(1);
    for i in 0..n_test {
        if pptr_start + i + 1 < prices.len() {
            targets[i] = prices[pptr_start + i + 1] - prices[pptr_start + i];
        }
    }

    // Do the test
    let mut sum = 0.0;
    for i in 0..n_test {
        let mut pred = 0.0;
        for ivar in 0..nvars {
            pred += cd.beta[ivar]
                * (data[i * nvars + ivar] - cd.xmeans[ivar])
                / cd.xscales[ivar];
        }
        pred = pred * cd.yscale + cd.ymean;

        if pred > 0.0 {
            sum += targets[i];
        } else if pred < 0.0 {
            sum -= targets[i];
        }
    }

    writeln!(
        fp_results,
        "\n\nOOS total return = {:.5} ({:.3} percent)",
        sum,
        100.0 * (sum.exp() - 1.0)
    )?;

    Ok(())
}