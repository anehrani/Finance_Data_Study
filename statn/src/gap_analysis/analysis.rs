const NGAPS: usize = 11;

pub fn initialize_gap_sizes() -> Vec<usize> {
    let mut gap_size = vec![0; NGAPS - 1];
    let mut k = 1;
    for gap in gap_size.iter_mut().take(NGAPS - 1) {
        *gap = k;
        k *= 2;
    }
    gap_size
}

pub fn gap_analyze(x: &[f64], thresh: f64, gap_size: &[usize]) -> Vec<usize> {
    let mut gap_count = vec![0; NGAPS];

    let mut count = 1;
    let mut above_below = if x[0] >= thresh { 1 } else { 0 };

    for i in 1..=x.len() {
        let new_above_below = if i == x.len() {
            1 - above_below
        } else if x[i] >= thresh {
            1
        } else {
            0
        };

        if new_above_below == above_below {
            count += 1;
        } else {
            let mut j = 0;
            while j < gap_size.len() {
                if count <= gap_size[j] {
                    break;
                }
                j += 1;
            }
            gap_count[j] += 1;
            count = 1;
            above_below = new_above_below;
        }
    }

    gap_count
}

pub fn print_gap_analysis(gap_size: &[usize], gap_count: &[usize], label: &str, lookback: usize) {
    println!("\n\nGap analysis for {} with lookback={}", label, lookback);
    println!("  Size   Count");

    for i in 0..NGAPS {
        if i < NGAPS - 1 {
            println!(" {:5} {:7}", gap_size[i], gap_count[i]);
        } else {
            println!(">{:5} {:7}", gap_size[NGAPS - 2], gap_count[i]);
        }
    }
}