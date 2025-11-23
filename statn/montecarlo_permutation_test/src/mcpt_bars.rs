use crate::random::Rand32M;

/// Compute optimal long-term rise and short-term drop thresholds
/// for a primitive mean reversion long-only system
pub fn opt_params(
    ncases: usize,
    lookback: usize,
    open: &[f64],
    close: &[f64],
) -> (f64, f64, f64, usize) {
    let mut best_perf = f64::NEG_INFINITY;
    let mut opt_rise = 0.0;
    let mut opt_drop = 0.0;
    let mut best_nlong = 0;
    
    for irise in 1..=50 {
        let rise_thresh = irise as f64 * 0.005;
        
        for idrop in 1..=50 {
            let drop_thresh = idrop as f64 * 0.0005;
            
            let mut total_return = 0.0;
            let mut nl = 0;
            
            for i in lookback..ncases - 2 {
                let rise = close[i] - close[i - lookback];
                let drop = close[i - 1] - close[i];
                
                let ret = if rise >= rise_thresh && drop >= drop_thresh {
                    nl += 1;
                    open[i + 2] - open[i + 1]
                } else {
                    0.0
                };
                
                total_return += ret;
            }
            
            if total_return > best_perf {
                best_perf = total_return;
                opt_rise = rise_thresh;
                opt_drop = drop_thresh;
                best_nlong = nl;
            }
        }
    }
    
    (best_perf, opt_rise, opt_drop, best_nlong)
}

/// Prepare for permutation by computing price changes
pub fn prepare_permute(
    nc: usize,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    rel_open: &mut [f64],
    rel_high: &mut [f64],
    rel_low: &mut [f64],
    rel_close: &mut [f64],
) {
    for icase in 1..nc {
        rel_open[icase - 1] = open[icase] - close[icase - 1];
        rel_high[icase - 1] = high[icase] - open[icase];
        rel_low[icase - 1] = low[icase] - open[icase];
        rel_close[icase - 1] = close[icase] - open[icase];
    }
}

/// Perform permutation by shuffling price changes and rebuilding prices
pub fn do_permute(
    nc: usize,
    preserve_oo: bool,
    open: &mut [f64],
    high: &mut [f64],
    low: &mut [f64],
    close: &mut [f64],
    rel_open: &mut [f64],
    rel_high: &mut [f64],
    rel_low: &mut [f64],
    rel_close: &mut [f64],
    rng: &mut Rand32M,
) {
    let preserve_offset = if preserve_oo { 1 } else { 0 };
    
    // Shuffle the close-to-open changes
    let mut i = nc - 1 - preserve_offset;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = j.min(i - 1);
        i -= 1;
        rel_open.swap(i + preserve_offset, j + preserve_offset);
    }
    
    // Shuffle the open-to-close changes
    let mut i = nc - 1 - preserve_offset;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = j.min(i - 1);
        i -= 1;
        rel_high.swap(i, j);
        rel_low.swap(i, j);
        rel_close.swap(i, j);
    }
    
    // Rebuild the prices using the shuffled changes
    for icase in 1..nc {
        open[icase] = close[icase - 1] + rel_open[icase - 1];
        high[icase] = open[icase] + rel_high[icase - 1];
        low[icase] = open[icase] + rel_low[icase - 1];
        close[icase] = open[icase] + rel_close[icase - 1];
    }
}

/// Run the MCPT bars analysis
pub fn run_mcpt_bars(
    lookback: usize,
    nreps: usize,
    mut open: Vec<f64>,
    mut high: Vec<f64>,
    mut low: Vec<f64>,
    mut close: Vec<f64>,
) -> Result<(), String> {
    let nprices = open.len();
    
    if nprices - lookback < 10 {
        return Err("Number of prices must be at least 10 greater than lookback".to_string());
    }
    
    println!("Market price history read");
    
    // Allocate work arrays
    let mut rel_open = vec![0.0; nprices];
    let mut rel_high = vec![0.0; nprices];
    let mut rel_low = vec![0.0; nprices];
    let mut rel_close = vec![0.0; nprices];
    
    let trend_per_return = (open[nprices - 1] - open[lookback + 1]) / (nprices - lookback - 2) as f64;
    
    // Prepare for permutation
    let eval_start = lookback;
    let eval_len = nprices - lookback;
    prepare_permute(
        eval_len,
        &open[eval_start..],
        &high[eval_start..],
        &low[eval_start..],
        &close[eval_start..],
        &mut rel_open,
        &mut rel_high,
        &mut rel_low,
        &mut rel_close,
    );
    
    let mut rng = Rand32M::default();
    let mut original = 0.0;
    let mut original_trend_component = 0.0;
    let mut original_nlong = 0;
    let mut count = 1;
    let mut mean_training_bias = 0.0;
    
    // Do MCPT
    for irep in 0..nreps {
        if irep > 0 {
            do_permute(
                eval_len,
                true,
                &mut open[eval_start..],
                &mut high[eval_start..],
                &mut low[eval_start..],
                &mut close[eval_start..],
                &mut rel_open,
                &mut rel_high,
                &mut rel_low,
                &mut rel_close,
                &mut rng,
            );
        }
        
        let (opt_return, opt_rise, opt_drop, nlong) = opt_params(nprices, lookback, &open, &close);
        let trend_component = nlong as f64 * trend_per_return;
        
        println!(
            "{:5}: Ret = {:.3}  Rise, drop= {:.4} {:.4}  NL={}  TrndComp={:.4}  TrnBias={:.4}",
            irep, opt_return, opt_rise, opt_drop, nlong, trend_component, opt_return - trend_component
        );
        
        if irep == 0 {
            original = opt_return;
            original_trend_component = trend_component;
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
    
    println!("\n{} prices were read, {} MCP replications with lookback = {}", 
             nprices, nreps, lookback);
    println!("\np-value for null hypothesis that system is worthless = {:.4}", 
             count as f64 / nreps as f64);
    println!("Total trend = {:.4}", open[nprices - 1] - open[lookback + 1]);
    println!("Original nlong = {}", original_nlong);
    println!("Original return = {:.4}", original);
    println!("Trend component = {:.4}", original_trend_component);
    println!("Training bias = {:.4}", mean_training_bias);
    println!("Skill = {:.4}", skill);
    println!("Unbiased return = {:.4}", unbiased_return);
    
    Ok(())
}
