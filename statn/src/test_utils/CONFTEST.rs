use std::env;
use std::io::{self, Read};

// These functions are assumed to be already implemented in Rust
extern "C" {
    fn unifrand() -> f64;
    fn qsortd(istart: i32, istop: i32, x: *mut f64);
    fn orderstat_tail(n: i32, q: f64, m: i32) -> f64;
    fn quantile_conf(n: i32, m: i32, conf: f64) -> f64;
}

fn main() {
    let args: Vec<String> = env::collect();
    
    let mut nsamps: i32;
    let mut lower_fail_rate: f64;
    let mut lower_bound_low_q: f64;
    let mut lower_bound_high_q: f64;
    let mut p_of_q: f64;
    
    // Process command line parameters
    if args.len() != 6 {
        println!("\nUsage: CONFTEST  nsamples fail_rate low_q high_q p_of_q");
        println!("  nsamples - Number of cases in each trial (at least 20)");
        println!("  fail_rate - Desired rate of failure for computed bound (smallish)");
        println!("  low_q - Worrisome failure rate below desired (< fail_rate)");
        println!("  high_q - Worrisome failure rate above desired (> fail_rate)");
        println!("  p_of_q - Small probability of failure; to get limits");
        std::process::exit(1);
    }
    
    nsamps = args[1].parse().expect("nsamples must be a number");
    lower_fail_rate = args[2].parse().expect("fail_rate must be a number");
    lower_bound_low_q = args[3].parse().expect("low_q must be a number");
    lower_bound_high_q = args[4].parse().expect("high_q must be a number");
    p_of_q = args[5].parse().expect("p_of_q must be a number");
    
    if nsamps < 20 || lower_bound_low_q >= lower_fail_rate || lower_bound_high_q <= lower_fail_rate {
        println!("\nUsage: CONFTEST  nsamples fail_rate low_q high_q p_of_q");
        println!("  nsamples - Number of cases in each trial (at least 20)");
        println!("  fail_rate - Desired rate of failure for computed bound (smallish)");
        println!("  low_q - Worrisome failure rate below desired (< fail_rate)");
        println!("  high_q - Worrisome failure rate above desired (> fail_rate)");
        println!("  p_of_q - Small probability of failure; to get limits");
        std::process::exit(1);
    }
    
    // Allocate memory and initialize
    let mut x: Vec<f64> = vec![0.0; nsamps as usize];
    
    let divisor = if 1_000_000 / nsamps < 2 { 2 } else { 1_000_000 / nsamps };
    
    let lower_bound_index = {
        let idx = (lower_fail_rate * (nsamps + 1) as f64) as i32 - 1;
        if idx < 0 { 0 } else { idx }
    };
    
    let lower_bound_low_theory = unsafe {
        1.0 - orderstat_tail(nsamps, lower_bound_low_q, lower_bound_index + 1)
    };
    
    let lower_bound_high_theory = unsafe {
        orderstat_tail(nsamps, lower_bound_high_q, lower_bound_index + 1)
    };
    
    let p_of_q_low_q = unsafe {
        quantile_conf(nsamps, lower_bound_index + 1, 1.0 - p_of_q)
    };
    
    let p_of_q_high_q = unsafe {
        quantile_conf(nsamps, lower_bound_index + 1, p_of_q)
    };
    
    println!(
        "\nnsamps={}  lower_fail_rate={:.3}  lower_bound_low_q={:.4}  p={:.4}  lower_bound_high_q={:.4}  p={:.4}",
        nsamps, lower_fail_rate, lower_bound_low_q, lower_bound_low_theory, lower_bound_high_q, lower_bound_high_theory
    );
    
    println!(
        "\np_of_q={:.3}  low_q={:.4}  high_q={:.4}",
        p_of_q, p_of_q_low_q, p_of_q_high_q
    );
    
    // Upper bound initialization
    let upper_bound_index = nsamps - 1 - lower_bound_index;
    let upper_fail_rate = lower_fail_rate;
    let upper_bound_low_q = 1.0 - lower_bound_high_q;
    let upper_bound_high_q = 1.0 - lower_bound_low_q;
    let upper_bound_low_theory = lower_bound_high_theory;
    let upper_bound_high_theory = lower_bound_low_theory;
    
    // Get ready to go
    println!("\n\nPress any key to begin...");
    let _ = io::stdin().read(&mut [0; 1]);
    
    let mut lower_bound_fail_above_count: i32 = 0;
    let mut lower_bound_fail_below_count: i32 = 0;
    let mut lower_bound_low_q_count: i32 = 0;
    let mut lower_bound_high_q_count: i32 = 0;
    let mut lower_p_of_q_low_count: i32 = 0;
    let mut lower_p_of_q_high_count: i32 = 0;
    let mut upper_bound_fail_above_count: i32 = 0;
    let mut upper_bound_fail_below_count: i32 = 0;
    let mut upper_bound_low_q_count: i32 = 0;
    let mut upper_bound_high_q_count: i32 = 0;
    let mut upper_p_of_q_low_count: i32 = 0;
    let mut upper_p_of_q_high_count: i32 = 0;
    
    // Here we go
    let mut itry: i32 = 1;
    loop {
        let f = 1.0 / itry as f64;
        
        if (itry % divisor) == 1 {
            println!("\n\n{}", itry);
        }
        
        // Generate this try's data
        for i in 0..nsamps as usize {
            unsafe {
                x[i] = unifrand();
            }
        }
        
        unsafe {
            qsortd(0, nsamps - 1, x.as_mut_ptr());
        }
        
        let lower_bound = x[lower_bound_index as usize];
        
        // Tally
        if lower_bound > lower_fail_rate {
            lower_bound_fail_above_count += 1;
        }
        
        if lower_bound < lower_fail_rate {
            lower_bound_fail_below_count += 1;
        }
        
        if lower_bound <= lower_bound_low_q {
            lower_bound_low_q_count += 1;
        }
        
        if lower_bound >= lower_bound_high_q {
            lower_bound_high_q_count += 1;
        }
        
        if lower_bound <= p_of_q_low_q {
            lower_p_of_q_low_count += 1;
        }
        
        if lower_bound >= p_of_q_high_q {
            lower_p_of_q_high_count += 1;
        }
        
        // Upper bound section
        let upper_bound = x[upper_bound_index as usize];
        
        if upper_bound > 1.0 - upper_fail_rate {
            upper_bound_fail_above_count += 1;
        }
        
        if upper_bound < 1.0 - upper_fail_rate {
            upper_bound_fail_below_count += 1;
        }
        
        if upper_bound <= upper_bound_low_q {
            upper_bound_low_q_count += 1;
        }
        
        if upper_bound >= upper_bound_high_q {
            upper_bound_high_q_count += 1;
        }
        
        if upper_bound <= 1.0 - p_of_q_high_q {
            upper_p_of_q_low_count += 1;
        }
        
        if upper_bound >= 1.0 - p_of_q_low_q {
            upper_p_of_q_high_count += 1;
        }
        
        // Print results so far
        if (itry % divisor) == 1 {
            println!(
                "\n\nLower bound fail above={:5.3}  Lower bound fail below={:5.3}",
                f * lower_bound_fail_above_count as f64,
                f * lower_bound_fail_below_count as f64
            );
            println!(
                "\nLower bound below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                f * lower_bound_low_q_count as f64,
                lower_bound_low_theory,
                f * lower_bound_high_q_count as f64,
                lower_bound_high_theory
            );
            println!(
                "\nLower p_of_q below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                f * lower_p_of_q_low_count as f64,
                p_of_q,
                f * lower_p_of_q_high_count as f64,
                p_of_q
            );
            println!(
                "\n\nUpper bound fail above={:5.3}  Upper bound fail below={:5.3}",
                f * upper_bound_fail_above_count as f64,
                f * upper_bound_fail_below_count as f64
            );
            println!(
                "\nUpper bound below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                f * upper_bound_low_q_count as f64,
                upper_bound_low_theory,
                f * upper_bound_high_q_count as f64,
                upper_bound_high_theory
            );
            println!(
                "\nUpper p_of_q below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                f * upper_p_of_q_low_count as f64,
                p_of_q,
                f * upper_p_of_q_high_count as f64,
                p_of_q
            );
        }
        
        // Check for keyboard interrupt (ESC key)
        if (itry % 10) == 1 {
            // Note: Rust doesn't have a direct equivalent to _kbhit()
            // This simplified version just breaks on user input
            // For a true non-blocking check, you'd need to use a crate like `crossterm`
            break;
        }
        
        itry += 1;
    }
}