//! Optimization criteria and parameter search

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptCriteria {
    MeanReturn = 0,
    ProfitFactor = 1,
    SharpeRatio = 2,
}

impl OptCriteria {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(OptCriteria::MeanReturn),
            1 => Some(OptCriteria::ProfitFactor),
            2 => Some(OptCriteria::SharpeRatio),
            _ => None,
        }
    }
}

/// Compute optimal short‑term and long‑term lookbacks for a moving‑average crossover system.
/// Returns (best_perf, best_short, best_long).
pub fn opt_params(
    criteria: OptCriteria,
    long_v_short: bool,  // true = long-only, false = short-only
    x: &[f64],
) -> (f64, usize, usize) {
    let ncases = x.len();
    let mut best_perf = f64::NEG_INFINITY;
    let mut ibestshort = 1usize;
    let mut ibestlong = 2usize;

    for ilong in 2..200 {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut win_sum = 1.0e-60;
            let mut lose_sum = 1.0e-60;
            let mut sum_squares = 1.0e-60;
            let mut n_trades = 0;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    // initial moving averages
                    short_sum = 0.0;
                    for j in 0..ishort {
                        short_sum += x[i - j];
                    }
                    long_sum = short_sum;
                    for j in ishort..ilong {
                        long_sum += x[i - j];
                    }
                } else {
                    // update moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }
                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;
                
                // Only trade in the specified direction
                let mut traded = false;
                let ret = if long_v_short && short_mean > long_mean {
                    // Long position
                    traded = true;
                    x[i + 1] - x[i]
                } else if !long_v_short && short_mean < long_mean {
                    // Short position
                    traded = true;
                    x[i] - x[i + 1]
                } else {
                    0.0
                };
                
                if traded {
                    n_trades += 1;
                    total_return += ret;
                    sum_squares += ret * ret;
                    if ret > 0.0 {
                        win_sum += ret;
                    } else {
                        lose_sum -= ret;
                    }
                }
            }

            let perf = match criteria {
                OptCriteria::MeanReturn => total_return / (n_trades as f64 + 1.0e-30),
                OptCriteria::ProfitFactor => win_sum / lose_sum,
                OptCriteria::SharpeRatio => {
                    let mean = total_return / (n_trades as f64 + 1.0e-30);
                    let var = sum_squares / (n_trades as f64 + 1.0e-30) - mean * mean;
                    let var = if var < 1.0e-20 { 1.0e-20 } else { var };
                    mean / var.sqrt()
                }
            };
            if perf > best_perf {
                best_perf = perf;
                ibestshort = ishort;
                ibestlong = ilong;
            }
        }
    }
    (best_perf, ibestshort, ibestlong)
}

/// Wrapper for backward compatibility - trades both directions (original TrnBias behavior)
pub fn opt_params_both_directions(
    criteria: OptCriteria,
    x: &[f64],
) -> (f64, usize, usize) {
    let ncases = x.len();
    let mut best_perf = f64::NEG_INFINITY;
    let mut ibestshort = 1usize;
    let mut ibestlong = 2usize;

    for ilong in 2..200 {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut win_sum = 1.0e-60;
            let mut lose_sum = 1.0e-60;
            let mut sum_squares = 1.0e-60;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    short_sum = 0.0;
                    for j in 0..ishort {
                        short_sum += x[i - j];
                    }
                    long_sum = short_sum;
                    for j in ishort..ilong {
                        long_sum += x[i - j];
                    }
                } else {
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }
                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;
                
                // Trade both directions (original behavior)
                let ret = if short_mean > long_mean {
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    x[i] - x[i + 1]
                } else {
                    0.0
                };
                
                total_return += ret;
                sum_squares += ret * ret;
                if ret > 0.0 {
                    win_sum += ret;
                } else {
                    lose_sum -= ret;
                }
            }

            let perf = match criteria {
                OptCriteria::MeanReturn => total_return / (ncases - ilong) as f64,
                OptCriteria::ProfitFactor => win_sum / lose_sum,
                OptCriteria::SharpeRatio => {
                    let mean = total_return / (ncases - ilong) as f64;
                    let var = sum_squares / (ncases - ilong) as f64 - mean * mean;
                    let std_dev = var.sqrt();
                    mean / (std_dev + 1.0e-8)
                }
            };
            if perf > best_perf {
                best_perf = perf;
                ibestshort = ishort;
                ibestlong = ilong;
            }
        }
    }
    (best_perf, ibestshort, ibestlong)
}

