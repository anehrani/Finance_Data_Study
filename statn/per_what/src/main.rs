use clap::Parser;
use anyhow::Result;


mod market;
mod system;

use system::{OptimizationCriterion, ReturnType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optimization criterion: 0=mean return; 1=profit factor; 2=Sharpe ratio
    #[arg(long, default_value_t = 1)]
    which_crit: i32,

    /// Include all bars in return, even those with no position? (0=no, 1=yes)
    #[arg(long, default_value_t = 0)]
    all_bars: i32,

    /// Return type for testing: 0=all bars; 1=bars with position open; 2=completed trades
    #[arg(long, default_value_t = 2)]
    ret_type: i32,

    /// Maximum moving-average lookback
    #[arg(long, default_value_t = 100)]
    max_lookback: usize,

    /// Number of bars in training set
    #[arg(long, default_value_t = 2000)]
    n_train: usize,

    /// Number of bars in test set
    #[arg(long, default_value_t = 1000)]
    n_test: usize,

    /// Market file (YYYYMMDD Price)
    #[arg(long)]
    filename: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let which_crit = OptimizationCriterion::from(args.which_crit);
    let all_bars = args.all_bars != 0;
    let ret_type = ReturnType::from(args.ret_type);
    let max_lookback = args.max_lookback;
    let n_train = args.n_train;
    let n_test = args.n_test;
    let filename = args.filename;

    if n_train < max_lookback + 10 {
        anyhow::bail!("n_train must be at least 10 greater than max_lookback");
    }

    println!("Reading market file...");
    let prices = market::read_market_prices(&filename)?;
    let nprices = prices.len();
    println!("Market price history read: {} prices", nprices);

    if n_train + n_test > nprices {
        anyhow::bail!("n_train + n_test must not exceed n_prices");
    }

    let mult = if which_crit == OptimizationCriterion::MeanReturn {
        println!("Mean return criterion will be multiplied by 25200 in all results");
        25200.0
    } else {
        1.0
    };

    let mut train_start = 0;
    let mut nret = 0;
    let mut all_returns = Vec::new();

    loop {
        // Train
        // We pass a slice of prices starting at train_start
        // The length of the slice should be n_train
        // But opt_params expects to be able to look back from the end of the slice?
        // No, opt_params iterates from max_lookback-1 to nprices-1.
        // So we should pass exactly the training set.
        
        let train_prices = &prices[train_start..train_start + n_train];
        
        let (lookback, thresh, last_pos, crit) = system::opt_params(
            which_crit,
            all_bars,
            train_prices,
            max_lookback,
        );

        println!(
            " IS at {}  Lookback={}  Thresh={:.3}  Crit={:.3}",
            train_start,
            lookback,
            thresh,
            mult * crit
        );

        let mut n = n_test;
        if n > nprices - train_start - n_train {
            n = nprices - train_start - n_train;
        }
        
        if n == 0 {
            break;
        }

        // Test
        // comp_return_full needs the full prices array (or at least enough context)
        // and the index where the test set starts.
        // The test set starts at train_start + n_train.
        
        let test_start_idx = train_start + n_train;
        
        let returns = system::comp_return_full(
            ret_type,
            &prices,
            test_start_idx,
            n,
            lookback,
            thresh,
            last_pos,
        );

        let n_returns = returns.len();
        nret += n_returns;
        all_returns.extend(returns);

        println!(
            "OOS testing {} from {} had {} returns, total={}",
            n, test_start_idx, n_returns, nret
        );

        // Advance fold window
        train_start += n;
        if train_start + n_train >= nprices {
            break;
        }
    }

    println!(
        "\n\nnprices={}  max_lookback={}  which_crit={:?}  all_bars={}  ret_type={:?}  n_train={}  n_test={}",
        nprices, max_lookback, args.which_crit, args.all_bars, args.ret_type, n_train, n_test
    );

    if nret > 0 {
        match which_crit {
            OptimizationCriterion::MeanReturn => {
                let sum: f64 = all_returns.iter().sum();
                let mean = sum / nret as f64;
                println!(
                    "\n\nOOS mean return per open-trade bar (times 25200) = {:.5}  nret={}",
                    25200.0 * mean,
                    nret
                );
            }
            OptimizationCriterion::ProfitFactor => {
                let mut win_sum = 1.0e-60;
                let mut lose_sum = 1.0e-60;
                for &r in &all_returns {
                    if r > 0.0 {
                        win_sum += r;
                    } else if r < 0.0 {
                        lose_sum -= r;
                    }
                }
                let pf = win_sum / lose_sum;
                println!("\n\nOOS profit factor = {:.5}  nret={}", pf, nret);
            }
            OptimizationCriterion::SharpeRatio => {
                let sum: f64 = all_returns.iter().sum();
                let sum_sq: f64 = all_returns.iter().map(|&r| r * r).sum();
                let mean = sum / nret as f64;
                let mean_sq = sum_sq / nret as f64;
                let mut variance = mean_sq - mean * mean;
                if variance < 1.0e-20 {
                    variance = 1.0e-20;
                }
                let sr = mean / variance.sqrt();
                println!("\n\nOOS raw Sharpe ratio = {:.5}  nret={}", sr, nret);
            }
        }
    } else {
        println!("\n\nNo returns generated.");
    }

    Ok(())
}
