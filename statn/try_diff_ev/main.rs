use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process;
use std::cell::RefCell;
use std::rc::Rc;

use statn::estimators::StocBias;
use statn::models::differential_evolution::diff_ev;

// Global/Shared data for the criterion function
struct MarketData {
    prices: Vec<f64>,
    max_lookback: usize,
}

/// Evaluate a thresholded moving-average crossover system
fn test_system(
    prices: &[f64],
    max_lookback: usize,
    long_term: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
    returns: Option<&mut [f64]>,
) -> (f64, i32) {
    let ncases = prices.len();
    let short_term = (0.01 * short_pct * long_term as f64) as usize;
    let short_term = short_term.max(1).min(long_term - 1);
    
    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    let mut sum = 0.0;
    let mut ntrades = 0;
    
    match returns {
        Some(ret_slice) => {
            let mut ret_idx = 0;
            for i in (max_lookback - 1)..(ncases - 1) {
                let mut short_mean = 0.0;
                for j in (i + 1 - short_term)..=i {
                    short_mean += prices[j];
                }
                
                let mut long_mean = short_mean;
                for j in (i + 1 - long_term)..(i + 1 - short_term) {
                     long_mean += prices[j];
                }
                
                short_mean /= short_term as f64;
                long_mean /= long_term as f64;
                
                let change = short_mean / long_mean - 1.0;
                
                let ret = if change > long_thresh {
                    ntrades += 1;
                    prices[i+1] - prices[i]
                } else if change < -short_thresh {
                    ntrades += 1;
                    prices[i] - prices[i+1]
                } else {
                    0.0
                };
                
                sum += ret;
                if ret_idx < ret_slice.len() {
                    ret_slice[ret_idx] = ret;
                    ret_idx += 1;
                }
            }
        }
        None => {
             for i in (max_lookback - 1)..(ncases - 1) {
                let mut short_mean = 0.0;
                for j in (i + 1 - short_term)..=i {
                    short_mean += prices[j];
                }
                
                let mut long_mean = short_mean;
                for j in (i + 1 - long_term)..(i + 1 - short_term) {
                     long_mean += prices[j];
                }
                
                short_mean /= short_term as f64;
                long_mean /= long_term as f64;
                
                let change = short_mean / long_mean - 1.0;
                
                let ret = if change > long_thresh {
                    ntrades += 1;
                    prices[i+1] - prices[i]
                } else if change < -short_thresh {
                    ntrades += 1;
                    prices[i] - prices[i+1]
                } else {
                    0.0
                };
                
                sum += ret;
            }
        }
    }

    (sum, ntrades)
}

/// Criterion function
fn criter(
    params: &[f64],
    mintrades: i32,
    data: &MarketData,
    stoc_bias: &mut Option<&mut StocBias>,
) -> f64 {
    let long_term = (params[0] + 1.0e-10) as usize;
    let short_pct = params[1];
    let short_thresh = params[2];
    let long_thresh = params[3];

    let (ret_val, ntrades) = if let Some(sb) = stoc_bias {
        let returns = sb.returns_mut();
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            Some(returns),
        )
    } else {
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            None,
        )
    };

    if let Some(sb) = stoc_bias {
        if ret_val > 0.0 {
            sb.process();
        }
    }

    if ntrades >= mintrades {
        ret_val
    } else {
        -1.0e20
    }
}

/// Compute and print parameter sensitivity curves
fn sensitivity<F>(
    mut criter: F,
    nvars: usize,
    nints: usize,
    npoints: usize,
    nres: usize,
    mintrades: i32,
    best: &[f64],
    low_bounds: &[f64],
    high_bounds: &[f64],
) -> io::Result<()>
where
    F: FnMut(&[f64], i32) -> f64,
{
    let mut fp = File::create("SENS.LOG")?;
    let mut params = best.to_vec();
    let mut vals = vec![0.0; npoints];

    for ivar in 0..nvars {
        // Reset params
        for i in 0..nvars {
            params[i] = best[i];
        }

        let mut maxval = -1.0e60;

        if ivar < nints {
            writeln!(fp, "\n\nSensitivity curve for integer parameter {} (optimum={})", ivar + 1, (best[ivar] + 1.0e-10) as i32)?;
            
            let label_frac = (high_bounds[ivar] - low_bounds[ivar] + 0.99999999) / (npoints as f64 - 1.0);
            
            for ipoint in 0..npoints {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                params[ivar] = ival as f64;
                vals[ipoint] = criter(&params, mintrades);
                if ipoint == 0 || vals[ipoint] > maxval {
                    maxval = vals[ipoint];
                }
            }
            
            let hist_frac = (nres as f64 + 0.9999999) / maxval.abs().max(1.0e-9);
            
            for ipoint in 0..npoints {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                write!(fp, "\n{:6}|", ival)?;
                let k = (vals[ipoint] * hist_frac) as i32;
                for _ in 0..k {
                    write!(fp, "*")?;
                }
            }

        } else {
            writeln!(fp, "\n\nSensitivity curve for real parameter {} (optimum={:.4})", ivar + 1, best[ivar])?;
            
            let label_frac = (high_bounds[ivar] - low_bounds[ivar]) / (npoints as f64 - 1.0);
            
            for ipoint in 0..npoints {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                params[ivar] = rval;
                vals[ipoint] = criter(&params, mintrades);
                if ipoint == 0 || vals[ipoint] > maxval {
                    maxval = vals[ipoint];
                }
            }
            
            let hist_frac = (nres as f64 + 0.9999999) / maxval.abs().max(1.0e-9);
            
            for ipoint in 0..npoints {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                write!(fp, "\n{:10.3}|", rval)?;
                let k = (vals[ipoint] * hist_frac) as i32;
                for _ in 0..k {
                    write!(fp, "*")?;
                }
            }
        }
    }
    
    Ok(())
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let (max_lookback, max_thresh, filename) = if args.len() == 4 {
        (
            args[1].parse::<usize>().unwrap_or(100),
            args[2].parse::<f64>().unwrap_or(100.0),
            args[3].clone(),
        )
    } else {
        println!("\nUsage: try_diff_ev  max_lookback  max_thresh  filename");
        println!("  max_lookback - Maximum moving-average lookback");
        println!("  max_thresh - Maximum fraction threshold times 10000");
        println!("  filename - name of market file (YYYYMMDD Price)");
        // Default values for testing if no args
        (100, 100.0, "test_data.txt".to_string())
    };

    // Read market prices
    let path = Path::new(&filename);
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("\n\nCannot open market history file {}: {}", filename, e);
            process::exit(1);
        }
    };
    let reader = io::BufReader::new(file);

    let mut prices = Vec::new();
    println!("\nReading market file...");

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                println!("\nError reading line {} of file {}", line_num + 1, filename);
                process::exit(1);
            }
        };
        
        if line.len() < 2 {
            continue;
        }

        // Parse: YYYYMMDD price1 price2 price3 price4
        // We want the last price (close price)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Take the last column as the close price
            if let Ok(price) = parts[parts.len() - 1].parse::<f64>() {
                if price > 0.0 {
                    prices.push(price.ln());
                }
            }
        }
    }

    println!("\nMarket price history read, {} prices", prices.len());
    
    if prices.len() <= max_lookback {
        println!("Not enough prices for max_lookback");
        process::exit(1);
    }

    let market_data = MarketData {
        prices,
        max_lookback,
    };

    let low_bounds = vec![2.0, 0.01, 0.0, 0.0];
    let high_bounds = vec![max_lookback as f64, 99.0, max_thresh, max_thresh];
    let mintrades = 20;

    // Initialize StocBias
    let mut stoc_bias = StocBias::new(market_data.prices.len() - max_lookback);
    if stoc_bias.is_none() {
        println!("Insufficient memory for StocBias");
        process::exit(1);
    }
    let mut stoc_bias = stoc_bias.unwrap();

    // We use unsafe to alias mutable reference to stoc_bias
    let sb_ptr = &mut stoc_bias as *mut StocBias;
    
    let criter_wrapper = |params: &[f64], mintrades: i32| -> f64 {
        let mut sb_ref = unsafe {
            if !sb_ptr.is_null() {
                Some(&mut *sb_ptr)
            } else {
                None
            }
        };
        criter(params, mintrades, &market_data, &mut sb_ref)
    };

    // Run diff_ev
    // We use a local version of diff_ev that handles StocBias via RefCell/Rc or just unsafe pointer if we inline it.
    // Since we can't easily modify the library function signature without breaking other things,
    // and we can't pass the same mutable reference to both the function and the closure,
    // we will use a simplified approach:
    // We will pass None to diff_ev for stoc_bias.
    // We will manually control stoc_bias collecting in the closure based on a shared flag?
    // No, we can't know when init ends.
    
    // Let's just copy diff_ev here and modify it to use the unsafe pointer or RefCell.
    // For brevity, I will assume I can modify the library in a future step if needed.
    // But for now, I will use the unsafe pointer trick and pass `None` to diff_ev, 
    // AND I will modify the closure to ALWAYS collect?
    // No, that would be wrong (bias would be wrong).
    
    // Actually, I can use `unsafe` to create a mutable reference to `stoc_bias` and pass it to `diff_ev`?
    // `diff_ev` takes `&mut Option<StocBias>`. It expects to hold the StocBias.
    // If I have `stoc_bias` on stack, I can pass `&mut Some(stoc_bias)`.
    // But then I can't access it in closure.
    
    // I will use a local `diff_ev` implementation.
    
    let result = local_diff_ev(
        criter_wrapper,
        4,
        1,
        100,
        10000,
        mintrades,
        10000000,
        300,
        0.2,
        0.2,
        0.3,
        &low_bounds,
        &high_bounds,
        true,
        sb_ptr, // Pass pointer to local_diff_ev
    );

    match result {
        Ok(params) => {
            println!("\n\nBest performance = {:.4}  Variables follow...", params[4]);
            for i in 0..4 {
                println!("\n  {:.4}", params[i]);
            }

            // Compute and print stochastic bias estimate
            let (is_mean, oos_mean, bias) = stoc_bias.compute();
            println!("\n\nVery rough estimates from differential evolution initialization...");
            println!("\n  In-sample mean = {:.4}", is_mean);
            println!("\n  Out-of-sample mean = {:.4}", oos_mean);
            println!("\n  Bias = {:.4}", bias);
            println!("\n  Expected = {:.4}", params[4] - bias);
            
            // Sensitivity
            // We need to reset stoc_bias or ensure criter doesn't use it?
            // C++: delete stoc_bias; stoc_bias = NULL;
            // We can just set our pointer to null?
            // Or just ignore it in sensitivity?
            // sensitivity calls criter. criter uses sb_ptr.
            // We should set sb_ptr to null? But sb_ptr is a copy.
            // We need to control the closure.
            // The closure checks `!sb_ptr.is_null()`.
            // We can't easily nullify the pointer inside the closure from here.
            // But we can set `stoc_bias.collecting` to false.
            stoc_bias.set_collecting(false);
            
            let _ = sensitivity(
                |p, m| criter(p, m, &market_data, &mut None), // Pass None for stoc_bias to disable collection
                4,
                1,
                30,
                80,
                mintrades,
                &params,
                &low_bounds,
                &high_bounds,
            );
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\n\nPress any key...");
    // std::io::stdin().read_line(&mut String::new()).unwrap();
}

// Local implementation of diff_ev to handle raw pointer for StocBias
use statn::core::matlib::paramcor::paramcor;
use statn::core::matlib::rands::unifrand;
use statn::estimators::brentmax::brentmax;
use statn::estimators::glob_max::glob_max;

fn local_diff_ev<F>(
    criter: F,
    nvars: usize,
    nints: usize,
    popsize: usize,
    overinit: usize,
    mut mintrades: i32,
    max_evals: usize,
    max_bad_gen: usize,
    mutate_dev: f64,
    pcross: f64,
    pclimb: f64,
    low_bounds: &[f64],
    high_bounds: &[f64],
    print_progress: bool,
    sb_ptr: *mut StocBias,
) -> Result<Vec<f64>, String>
where
    F: Fn(&[f64], i32) -> f64 + Copy,
{
    let dim = nvars + 1;
    let mut pop1 = vec![0.0; dim * popsize];
    let mut pop2 = vec![0.0; dim * popsize];
    let mut best = vec![0.0; dim];

    let mut failures = 0;
    let mut n_evals = 0;

    // Turn on StocBias data collection
    unsafe {
        if !sb_ptr.is_null() {
            (*sb_ptr).set_collecting(true);
        }
    }

    let mut grand_best = -1.0e60;
    let mut worstf = 1.0e60;
    let mut avgf = 0.0;

    let mut ind = 0;
    
    while ind < popsize + overinit {
        let value = {
            let popptr_slice = if ind < popsize {
                &mut pop1[ind * dim..(ind + 1) * dim]
            } else {
                &mut pop2[0..dim]
            };

            for i in 0..nvars {
                if i < nints {
                    popptr_slice[i] = low_bounds[i]
                        + (unifrand() * (high_bounds[i] - low_bounds[i] + 1.0)).floor();
                    if popptr_slice[i] > high_bounds[i] {
                        popptr_slice[i] = high_bounds[i];
                    }
                } else {
                    popptr_slice[i] = low_bounds[i] + (unifrand() * (high_bounds[i] - low_bounds[i]));
                }
            }

            let val = criter(&popptr_slice[0..nvars], mintrades);
            popptr_slice[nvars] = val;
            val
        };
        
        n_evals += 1;

        let mut current_ind = vec![0.0; dim];
        if ind < popsize {
            current_ind.copy_from_slice(&pop1[ind * dim..(ind + 1) * dim]);
        } else {
            current_ind.copy_from_slice(&pop2[0..dim]);
        }

        if ind == 0 {
            grand_best = value;
            worstf = value;
            avgf = value;
            best.copy_from_slice(&current_ind);
        }

        if value <= 0.0 {
            if n_evals > max_evals {
                 break; 
            }
            
            failures += 1;
            if failures >= 500 {
                failures = 0;
                mintrades = mintrades * 9 / 10;
                if mintrades < 1 {
                    mintrades = 1;
                }
            }
            continue; 
        } else {
            failures = 0;
        }

        if value > grand_best {
            best.copy_from_slice(&current_ind);
            grand_best = value;
        }

        if value < worstf {
            worstf = value;
        }

        avgf += value;

        if print_progress {
            let avg = if ind < popsize {
                avgf / (ind as f64 + 1.0)
            } else {
                avgf / popsize as f64
            };
            print!(
                "\n{}: Val={:.4} Best={:.4} Worst={:.4} Avg={:.4}  (fail rate={:.1})",
                ind,
                value,
                grand_best,
                worstf,
                avg,
                n_evals as f64 / (ind as f64 + 1.0)
            );
            for i in 0..nvars {
                print!(" {:.4}", current_ind[i]);
            }
        }

        if ind >= popsize {
            avgf = 0.0;
            let mut min_idx = 0;
            let mut current_worst = 1.0e60;

            for i in 0..popsize {
                let dtemp = pop1[i * dim + nvars];
                avgf += dtemp;
                if i == 0 || dtemp < current_worst {
                    min_idx = i;
                    current_worst = dtemp;
                }
            }
            worstf = current_worst;

            if value > worstf {
                let dest = &mut pop1[min_idx * dim..(min_idx + 1) * dim];
                dest.copy_from_slice(&current_ind);
                avgf += value - worstf;
            }
        }

        ind += 1;
    }
    
    if n_evals > max_evals && grand_best <= 0.0 {
         return Ok(best);
    }

    // Turn off StocBias data collection
    unsafe {
        if !sb_ptr.is_null() {
            (*sb_ptr).set_collecting(false);
        }
    }

    let mut ibest = 0;
    let mut value = pop1[nvars];
    for ind in 1..popsize {
        let val = pop1[ind * dim + nvars];
        if val > value {
            value = val;
            ibest = ind;
        }
    }
    
    let mut generation = 1;
    let mut bad_generations = 0;
    let mut n_tweaked = 0;
    
    loop {
        worstf = 1.0e60;
        avgf = 0.0;
        let mut improved = false;

        for ind in 0..popsize {
            let mut i;
            let mut j;
            let mut k;
            
            loop {
                i = (unifrand() * popsize as f64) as usize;
                if i < popsize && i != ind { break; }
            }
            loop {
                j = (unifrand() * popsize as f64) as usize;
                if j < popsize && j != ind && j != i { break; }
            }
            loop {
                k = (unifrand() * popsize as f64) as usize;
                if k < popsize && k != ind && k != i && k != j { break; }
            }

            let p1_idx = ind * dim;
            let p2_idx = i * dim;
            let d1_idx = j * dim;
            let d2_idx = k * dim;
            let dest_idx = ind * dim;
            
            let start_param = (unifrand() * nvars as f64) as usize;
            let mut used_mutated = false;
            
            let mut curr_param_idx = (unifrand() * nvars as f64) as usize;
            if curr_param_idx >= nvars { curr_param_idx = nvars - 1; }
            
            for v in (0..nvars).rev() {
                 let should_mutate = (v == 0 && !used_mutated) || (unifrand() < pcross);
                 
                 if should_mutate {
                     let val = pop1[p2_idx + curr_param_idx] + mutate_dev * (pop1[d1_idx + curr_param_idx] - pop1[d2_idx + curr_param_idx]);
                     pop2[dest_idx + curr_param_idx] = val;
                     used_mutated = true;
                 } else {
                     pop2[dest_idx + curr_param_idx] = pop1[p1_idx + curr_param_idx];
                 }
                 
                 curr_param_idx = (curr_param_idx + 1) % nvars;
            }
            
            ensure_legal(nvars, nints, low_bounds, high_bounds, &mut pop2[dest_idx..dest_idx+nvars]);
            
            let mut child_val = criter(&pop2[dest_idx..dest_idx+nvars], mintrades);
            
            let parent_val = pop1[p1_idx + nvars];
            
            if child_val > parent_val {
                pop2[dest_idx + nvars] = child_val;
                if child_val > grand_best {
                    grand_best = child_val;
                    best.copy_from_slice(&pop2[dest_idx..dest_idx+dim]);
                    ibest = ind;
                    n_tweaked = 0;
                    improved = true;
                }
            } else {
                for x in 0..dim {
                    pop2[dest_idx + x] = pop1[p1_idx + x];
                }
                child_val = parent_val;
            }
            
            if pclimb > 0.0 && ((ind == ibest && n_tweaked < nvars) || (unifrand() < pclimb)) {
                let k_var = if ind == ibest {
                    n_tweaked += 1;
                    generation % nvars
                } else {
                    (unifrand() * nvars as f64) as usize
                };
                
                let k_var = if k_var >= nvars { nvars - 1 } else { k_var };
                
                if k_var < nints {
                    let ibase = pop2[dest_idx + k_var] as i32;
                    let ilow = low_bounds[k_var] as i32;
                    let ihigh = high_bounds[k_var] as i32;
                    let mut success = false;
                    
                    if print_progress {
                         print!("\nCriterion maximization of individual {} integer variable {} from {} = {:.6}", ind, k_var, ibase, child_val);
                    }
                    
                    let mut current_best_int = ibase;
                    let mut current_best_val = child_val;
                    
                    let mut ivar = ibase;
                    while ivar < ihigh {
                        ivar += 1;
                        pop2[dest_idx + k_var] = ivar as f64;
                        let test_val = criter(&pop2[dest_idx..dest_idx+nvars], mintrades);
                        if print_progress { print!("\n  {} = {:.6}", ivar, test_val); }
                        
                        if test_val > current_best_val {
                            current_best_val = test_val;
                            current_best_int = ivar;
                            success = true;
                        } else {
                            break;
                        }
                    }
                    
                    pop2[dest_idx + k_var] = current_best_int as f64;
                    
                    if !success {
                        ivar = ibase;
                        while ivar > ilow {
                            ivar -= 1;
                            pop2[dest_idx + k_var] = ivar as f64;
                            let test_val = criter(&pop2[dest_idx..dest_idx+nvars], mintrades);
                            if print_progress { print!("\n  {} = {:.6}", ivar, test_val); }
                            
                            if test_val > current_best_val {
                                current_best_val = test_val;
                                current_best_int = ivar;
                                success = true;
                            } else {
                                break;
                            }
                        }
                        pop2[dest_idx + k_var] = current_best_int as f64;
                    }
                    
                    child_val = current_best_val;
                    
                    if print_progress {
                        if success {
                            print!("\nSuccess at {:.0} = {:.6}", pop2[dest_idx + k_var], child_val);
                        } else {
                            print!("\nNo success at {:.0} = {:.6}", pop2[dest_idx + k_var], child_val);
                        }
                    }

                } else {
                    let local_base = pop2[dest_idx + k_var];
                    let old_value = child_val;
                    
                    if print_progress {
                        print!("\nCriterion maximization of individual {} real variable {} from {:.5} = {:.6}", ind, k_var, local_base, child_val);
                    }
                    
                    let mut lower = local_base - 0.1 * (high_bounds[k_var] - low_bounds[k_var]);
                    let mut upper = local_base + 0.1 * (high_bounds[k_var] - low_bounds[k_var]);
                    
                    if lower < low_bounds[k_var] {
                        lower = low_bounds[k_var];
                        upper = low_bounds[k_var] + 0.2 * (high_bounds[k_var] - low_bounds[k_var]);
                    }
                    if upper > high_bounds[k_var] {
                        upper = high_bounds[k_var];
                        lower = high_bounds[k_var] - 0.2 * (high_bounds[k_var] - low_bounds[k_var]);
                    }
                    
                    let mut temp_params = pop2[dest_idx..dest_idx+nvars].to_vec();
                    
                    let c_func = |param: f64| -> f64 {
                        let mut my_params = temp_params.clone();
                        my_params[k_var] = param;
                        let penalty = ensure_legal(nvars, nints, low_bounds, high_bounds, &mut my_params);
                        criter(&my_params, mintrades) - penalty
                    };
                    
                    let mut x1 = 0.0;
                    let mut y1 = 0.0;
                    let mut x2 = local_base;
                    let mut y2 = old_value;
                    let mut x3 = 0.0;
                    let mut y3 = 0.0;
                    
                    glob_max(lower, upper, 7, false, c_func, &mut x1, &mut y1, &mut x2, &mut y2, &mut x3, &mut y3);
                    brentmax(5, 1.0e-8, 0.0001, c_func, &mut x1, &mut x2, &mut x3, y2);
                    
                    pop2[dest_idx + k_var] = x2;
                    ensure_legal(nvars, nints, low_bounds, high_bounds, &mut pop2[dest_idx..dest_idx+nvars]);
                    child_val = criter(&pop2[dest_idx..dest_idx+nvars], mintrades);
                    
                    if child_val > old_value {
                        pop2[dest_idx + nvars] = child_val;
                        if print_progress {
                            print!("\nSuccess at {:.5} = {:.6}", pop2[dest_idx + k_var], child_val);
                        }
                    } else {
                        pop2[dest_idx + k_var] = local_base;
                        child_val = old_value;
                        if print_progress {
                            print!("\nNo success at {:.5} = {:.6}", pop2[dest_idx + k_var], child_val);
                        }
                    }
                    
                    if child_val > grand_best {
                        grand_best = child_val;
                        best.copy_from_slice(&pop2[dest_idx..dest_idx+dim]);
                        ibest = ind;
                        n_tweaked = 0;
                        improved = true;
                    }
                }
            }
            
            if child_val < worstf {
                worstf = child_val;
            }
            avgf += child_val;
            
        }

        if print_progress {
            print!("\nGen {} Best={:.4} Worst={:.4} Avg={:.4}", generation, grand_best, worstf, avgf / popsize as f64);
            for i in 0..nvars {
                print!(" {:.4}", best[i]);
            }
        }
        
        if !improved {
            bad_generations += 1;
            if bad_generations > max_bad_gen {
                break;
            }
        } else {
            bad_generations = 0;
        }
        
        std::mem::swap(&mut pop1, &mut pop2);
        
        generation += 1;
    }
    
    let _ = paramcor(&pop1, nvars);
    
    Ok(best)
}

fn ensure_legal(
    nvars: usize,
    nints: usize,
    low_bounds: &[f64],
    high_bounds: &[f64],
    params: &mut [f64],
) -> f64 {
    let mut penalty = 0.0;
    for i in 0..nvars {
        if i < nints {
            if params[i] >= 0.0 {
                params[i] = (params[i] + 0.5).floor();
            } else {
                params[i] = -(0.5 - params[i]).floor();
            }
        }
        
        if params[i] > high_bounds[i] {
            penalty += 1.0e10 * (params[i] - high_bounds[i]);
            params[i] = high_bounds[i];
        }
        if params[i] < low_bounds[i] {
            penalty += 1.0e10 * (low_bounds[i] - params[i]);
            params[i] = low_bounds[i];
        }
    }
    penalty
}
