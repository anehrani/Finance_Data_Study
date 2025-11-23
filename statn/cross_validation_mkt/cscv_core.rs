use crate::criter::criter;

/// Combinatorially symmetric cross validation core routine
/// 
/// # Arguments
/// * `ncases` - Number of columns in returns matrix (change fastest)
/// * `n_systems` - Number of rows (competitors); should be large enough to reduce granularity
/// * `n_blocks` - Number of blocks (even!) into which the cases will be partitioned
/// * `returns` - N_systems by ncases matrix of returns, case changing fastest
/// 
/// # Returns
/// Probability that the best in-sample system is at or below the median out-of-sample performance
pub fn cscvcore(
    ncases: usize,
    n_systems: usize,
    n_blocks: usize,
    returns: &[f64],
) -> f64 {
    // Make sure n_blocks is even
    let n_blocks = (n_blocks / 2) * 2;
    
    // Allocate work vectors
    let mut indices = vec![0; n_blocks];
    let mut lengths = vec![0; n_blocks];
    let mut flags = vec![0; n_blocks];
    let mut work = vec![0.0; ncases];
    let mut is_crits = vec![0.0; n_systems];
    let mut oos_crits = vec![0.0; n_systems];
    
    // Find the starting index and length of each of the n_blocks submatrices
    let mut istart = 0;
    for i in 0..n_blocks {
        indices[i] = istart;
        lengths[i] = (ncases - istart) / (n_blocks - i);
        istart += lengths[i];
    }
    
    // Initialize flags: first half are training set (1), second half are test set (0)
    for i in 0..(n_blocks / 2) {
        flags[i] = 1;
    }
    for i in (n_blocks / 2)..n_blocks {
        flags[i] = 0;
    }
    
    let mut nless = 0; // Count of times OOS of best <= median OOS
    let mut ncombo = 0; // Count of combinations
    
    // Main loop processes all combinations of blocks
    loop {
        // Compute training-set (IS) criterion for each candidate system
        for isys in 0..n_systems {
            let mut n = 0;
            for ic in 0..n_blocks {
                if flags[ic] == 1 {
                    // This block is in the training set
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }
            is_crits[isys] = criter(&work[0..n]);
        }
        
        // Compute OOS criterion for each candidate system
        for isys in 0..n_systems {
            let mut n = 0;
            for ic in 0..n_blocks {
                if flags[ic] == 0 {
                    // This block is in the OOS set
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }
            oos_crits[isys] = criter(&work[0..n]);
        }
        
        // Determine the relative rank within OOS of the system which had best IS performance
        let mut best_is = is_crits[0];
        let mut ibest = 0;
        for isys in 1..n_systems {
            if is_crits[isys] > best_is {
                best_is = is_crits[isys];
                ibest = isys;
            }
        }
        
        let best_oos = oos_crits[ibest];
        let mut n = 0;
        for isys in 0..n_systems {
            if isys == ibest || best_oos >= oos_crits[isys] {
                n += 1;
            }
        }
        
        let rel_rank = n as f64 / (n_systems + 1) as f64;
        
        if rel_rank <= 0.5 {
            nless += 1;
        }
        
        ncombo += 1;
        
        // Move to the next combination
        let mut iradix = 0;
        let mut found = false;
        let mut n_flags = 0;
        
        for ir in 0..(n_blocks - 1) {
            if flags[ir] == 1 {
                n_flags += 1;
                if flags[ir + 1] == 0 {
                    flags[ir] = 0;
                    flags[ir + 1] = 1;
                    
                    // Reset everything below this change point
                    let mut reset_count = n_flags - 1;
                    for i in 0..ir {
                        if reset_count > 0 {
                            flags[i] = 1;
                            reset_count -= 1;
                        } else {
                            flags[i] = 0;
                        }
                    }
                    
                    iradix = ir;
                    found = true;
                    break;
                }
            }
        }
        
        if !found || iradix == n_blocks - 1 {
            break;
        }
    }
    
    nless as f64 / ncombo as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cscvcore_basic() {
        // Create a simple returns matrix: 4 systems, 8 cases
        let n_systems = 4;
        let ncases = 8;
        let mut returns = vec![0.0; n_systems * ncases];
        
        // Fill with some test data
        for i in 0..n_systems {
            for j in 0..ncases {
                returns[i * ncases + j] = (i as f64 + j as f64) / 10.0;
            }
        }
        
        let prob = cscvcore(ncases, n_systems, 4, &returns);
        
        // Probability should be between 0 and 1
        assert!(prob >= 0.0 && prob <= 1.0);
    }
}
