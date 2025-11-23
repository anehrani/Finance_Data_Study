use std::env;
use std::process;
use std::io::{self, Write};
use rand::Rng;
use stats::{orderstat_tail, quantile_conf};

fn main() {
    let args: Vec<String> = env::args().collect();

    let (nsamps, lower_fail_rate, lower_bound_low_q, lower_bound_high_q, p_of_q) = if args.len() == 6 {
        (
            args[1].parse::<usize>().expect("Invalid nsamples"),
            args[2].parse::<f64>().expect("Invalid fail_rate"),
            args[3].parse::<f64>().expect("Invalid low_q"),
            args[4].parse::<f64>().expect("Invalid high_q"),
            args[5].parse::<f64>().expect("Invalid p_of_q"),
        )
    } else if args.len() == 1 {
        // Default values
        (100000, 0.1, 0.0975, 0.101, 0.01)
    } else {
        println!("\nUsage: conftest nsamples fail_rate low_q high_q p_of_q");
        println!("  nsamples - Number of cases in each trial (at least 20)");
        println!("  fail_rate - Desired rate of failure for computed bound (smallish)");
        println!("  low_q - Worrisome failure rate below desired (< fail_rate)");
        println!("  high_q - Worrisome failure rate above desired (> fail_rate)");
        println!("  p_of_q - Small probability of failure; to get limits");
        process::exit(1);
    };

    if nsamps < 20 || lower_bound_low_q >= lower_fail_rate || lower_bound_high_q <= lower_fail_rate {
        println!("\nUsage: conftest nsamples fail_rate low_q high_q p_of_q");
        println!("  nsamples - Number of cases in each trial (at least 20)");
        println!("  fail_rate - Desired rate of failure for computed bound (smallish)");
        println!("  low_q - Worrisome failure rate below desired (< fail_rate)");
        println!("  high_q - Worrisome failure rate above desired (> fail_rate)");
        println!("  p_of_q - Small probability of failure; to get limits");
        process::exit(1);
    }

    let divisor = (1000000 / nsamps).max(2);

    let lower_bound_index = (lower_fail_rate * (nsamps as f64 + 1.0)) as isize - 1;
    let lower_bound_index = lower_bound_index.max(0) as usize;

    let lower_bound_low_theory = 1.0 - orderstat_tail(nsamps as i32, lower_bound_low_q, (lower_bound_index + 1) as i32);
    let lower_bound_high_theory = orderstat_tail(nsamps as i32, lower_bound_high_q, (lower_bound_index + 1) as i32);

    let p_of_q_low_q = quantile_conf(nsamps as i32, (lower_bound_index + 1) as i32, 1.0 - p_of_q);
    let p_of_q_high_q = quantile_conf(nsamps as i32, (lower_bound_index + 1) as i32, p_of_q);

    println!("\nnsamps={}  lower_fail_rate={:.3}  lower_bound_low_q={:.4}  p={:.4}  lower_bound_high_q={:.4}  p={:.4}",
             nsamps, lower_fail_rate, lower_bound_low_q, lower_bound_low_theory, lower_bound_high_q, lower_bound_high_theory);

    println!("\np_of_q={:.3}  low_q={:.4}  high_q={:.4}", p_of_q, p_of_q_low_q, p_of_q_high_q);

    let upper_bound_index = nsamps - 1 - lower_bound_index;
    let upper_fail_rate = lower_fail_rate;
    let upper_bound_low_q = 1.0 - lower_bound_high_q;
    let upper_bound_high_q = 1.0 - lower_bound_low_q;
    let upper_bound_low_theory = lower_bound_high_theory;
    let upper_bound_high_theory = lower_bound_low_theory;

    if env::var("CONFTEST_MAX_ITERS").is_err() {
        println!("\n\nPress Enter to begin...");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }

    let mut lower_bound_fail_above_count = 0;
    let mut lower_bound_fail_below_count = 0;
    let mut lower_bound_low_q_count = 0;
    let mut lower_bound_high_q_count = 0;
    let mut lower_p_of_q_low_count = 0;
    let mut lower_p_of_q_high_count = 0;

    let mut upper_bound_fail_above_count = 0;
    let mut upper_bound_fail_below_count = 0;
    let mut upper_bound_low_q_count = 0;
    let mut upper_bound_high_q_count = 0;
    let mut upper_p_of_q_low_count = 0;
    let mut upper_p_of_q_high_count = 0;

    let mut rng = rand::thread_rng();
    let mut x = vec![0.0; nsamps];

    let mut itry = 1;
    loop {
        let f = 1.0 / itry as f64;

        if itry % divisor == 1 {
            print!("\n\n{}", itry);
            io::stdout().flush().unwrap();
        }

        for i in 0..nsamps {
            x[i] = rng.gen();
        }
        x.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let lower_bound = x[lower_bound_index];

        if lower_bound > lower_fail_rate { lower_bound_fail_above_count += 1; }
        if lower_bound < lower_fail_rate { lower_bound_fail_below_count += 1; }
        if lower_bound <= lower_bound_low_q { lower_bound_low_q_count += 1; }
        if lower_bound >= lower_bound_high_q { lower_bound_high_q_count += 1; }
        if lower_bound <= p_of_q_low_q { lower_p_of_q_low_count += 1; }
        if lower_bound >= p_of_q_high_q { lower_p_of_q_high_count += 1; }

        let upper_bound = x[upper_bound_index];

        if upper_bound > 1.0 - upper_fail_rate { upper_bound_fail_above_count += 1; }
        if upper_bound < 1.0 - upper_fail_rate { upper_bound_fail_below_count += 1; }
        if upper_bound <= upper_bound_low_q { upper_bound_low_q_count += 1; }
        if upper_bound >= upper_bound_high_q { upper_bound_high_q_count += 1; }
        if upper_bound <= 1.0 - p_of_q_high_q { upper_p_of_q_low_count += 1; }
        if upper_bound >= 1.0 - p_of_q_low_q { upper_p_of_q_high_count += 1; }

        if itry % divisor == 1 {
             println!("\n\nLower bound fail above={:5.3}  Lower bound fail below={:5.3}",
                       f * lower_bound_fail_above_count as f64, f * lower_bound_fail_below_count as f64);
             println!("Lower bound below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                       f * lower_bound_low_q_count as f64, lower_bound_low_theory, f * lower_bound_high_q_count as f64, lower_bound_high_theory);
             println!("Lower p_of_q below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                       f * lower_p_of_q_low_count as f64, p_of_q, f * lower_p_of_q_high_count as f64, p_of_q);

             println!("\n\nUpper bound fail above={:5.3}  Upper bound fail below={:5.3}",
                       f * upper_bound_fail_above_count as f64, f * upper_bound_fail_below_count as f64);
             println!("Upper bound below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                       f * upper_bound_low_q_count as f64, upper_bound_low_theory, f * upper_bound_high_q_count as f64, upper_bound_high_theory);
             println!("Upper p_of_q below lower limit={:5.4}  theory p={:.4}  above upper limit={:5.4}  theory p={:.4}",
                       f * upper_p_of_q_low_count as f64, p_of_q, f * upper_p_of_q_high_count as f64, p_of_q);
        }

        itry += 1;

        if let Ok(max_iters) = env::var("CONFTEST_MAX_ITERS") {
            if let Ok(limit) = max_iters.parse::<usize>() {
                if itry > limit {
                    break;
                }
            }
        }
    }
}
