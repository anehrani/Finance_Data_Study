use crate::random::Rand32M;

/// Compute optimal short-term and long-term lookbacks
/// for a primitive moving-average crossover system
pub fn opt_params(
    ncases: usize,
    max_lookback: usize,
    x: &[f64],
) -> (f64, usize, usize, usize, usize) {
    let mut best_perf = f64::NEG_INFINITY;
    let mut best_short_term = 0;
    let mut best_long_term = 0;
    let mut best_nlong = 0;
    let mut best_nshort = 0;
    
    for ilong in 2..=max_lookback {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut nl = 0;
            let mut ns = 0;
            
            let mut short_sum = 0.0;
            let mut long_sum = 0.0;
            
            for i in max_lookback - 1..ncases - 1 {
                if i == max_lookback - 1 {
                    // Initialize moving averages for first case
                    short_sum = 0.0;
                    for j in (i - ishort + 1..=i).rev() {
                        short_sum += x[j];
                    }
                    long_sum = short_sum;
                    for j in (i - ilong + 1..i - ishort + 1).rev() {
                        long_sum += x[j];
                    }
                } else {
                    // Update moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }
                
                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;
                
                let ret = if short_mean > long_mean {
                    nl += 1;
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    ns += 1;
                    x[i] - x[i + 1]
                } else {
                    0.0
                };
                
                total_return += ret;
            }
            
            if total_return > best_perf {
                best_perf = total_return;
                best_short_term = ishort;
                best_long_term = ilong;
                best_nlong = nl;
                best_nshort = ns;
            }
        }
    }
    
    (best_perf, best_short_term, best_long_term, best_nshort, best_nlong)
}

/// Prepare for permutation by computing price changes
pub fn prepare_permute(nc: usize, data: &[f64], changes: &mut [f64]) {
    for icase in 1..nc {
        changes[icase - 1] = data[icase] - data[icase - 1];
    }
}

/// Perform permutation by shuffling price changes and rebuilding prices
pub fn do_permute(nc: usize, data: &mut [f64], changes: &mut [f64], rng: &mut Rand32M) {
    // Shuffle the changes (excluding the first case)
    let mut i = nc - 1;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = j.min(i - 1);
        i -= 1;
        changes.swap(i, j);
    }
    
    // Rebuild the prices using the shuffled changes
    for icase in 1..nc {
        data[icase] = data[icase - 1] + changes[icase - 1];
    }
}

/// Run the MCPT trend analysis
pub fn run_mcpt_trend(
    max_lookback: usize,
    nreps: usize,
    mut prices: Vec<f64>,
) -> Result<(), String> {
    let nprices = prices.len();
    
    if nprices - max_lookback < 10 {
        return Err("Number of prices must be at least 10 greater than max_lookback".to_string());
    }
    
    println!("Market price history read");
    
    // Allocate work array
    let mut changes = vec![0.0; nprices];
    
    let trend_per_return = (prices[nprices - 1] - prices[max_lookback - 1]) / (nprices - max_lookback) as f64;
    
    // Prepare for permutation
    let eval_start = max_lookback - 1;
    let eval_len = nprices - max_lookback + 1;
    prepare_permute(eval_len, &prices[eval_start..], &mut changes);
    
    let mut rng = Rand32M::default();
    let mut original = 0.0;
    let mut original_trend_component = 0.0;
    let mut original_nshort = 0;
    let mut original_nlong = 0;
    let mut count = 1;
    let mut mean_training_bias = 0.0;
    
    // Do MCPT
    for irep in 0..nreps {
        if irep > 0 {
            do_permute(eval_len, &mut prices[eval_start..], &mut changes, &mut rng);
        }
        
        let (opt_return, short_lookback, long_lookback, nshort, nlong) = 
            opt_params(nprices, max_lookback, &prices);
        let trend_component = (nlong as f64 - nshort as f64) * trend_per_return;
        
        println!(
            "{:5}: Ret = {:.3}  Lookback={} {}  NS, NL={} {}  TrndComp={:.4}  TrnBias={:.4}",
            irep, opt_return, short_lookback, long_lookback, nshort, nlong, 
            trend_component, opt_return - trend_component
        );
        
        if irep == 0 {
            original = opt_return;
            original_trend_component = trend_component;
            original_nshort = nshort;
            original_nlong = nlong;
        } else {
            let training_bias = opt_return - trend_component;
            mean_training_bias += training_bias;
            if opt_return >= original {
                count += 1;
            }
        }
    }
    
    mean_training_bias /= (nreps - 1) as f64;
    let unbiased_return = original - mean_training_bias;
    let skill = unbiased_return - original_trend_component;
    
    println!("\n{} prices were read, {} MCP replications with max lookback = {}", 
             nprices, nreps, max_lookback);
    println!("\np-value for null hypothesis that system is worthless = {:.4}", 
             count as f64 / nreps as f64);
    println!("Total trend = {:.4}", prices[nprices - 1] - prices[max_lookback - 1]);
    println!("Original nshort = {}", original_nshort);
    println!("Original nlong = {}", original_nlong);
    println!("Original return = {:.4}", original);
    println!("Trend component = {:.4}", original_trend_component);
    println!("Training bias = {:.4}", mean_training_bias);
    println!("Skill = {:.4}", skill);
    println!("Unbiased return = {:.4}", unbiased_return);
    
    Ok(())
}
