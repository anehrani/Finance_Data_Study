/// Criterion function for CSCV - calculates mean of returns
fn criter(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    returns.iter().sum::<f64>() / returns.len() as f64
}

/// Computationally symmetric cross validation core routine
///
/// # Arguments
/// * `ncases` - Number of columns in returns matrix (change fastest)
/// * `n_systems` - Number of rows (competitors); should be large enough to reduce granularity
/// * `n_blocks_input` - Number of blocks (even!) into which the cases will be partitioned
/// * `returns` - N_systems by ncases matrix of returns, case changing fastest
/// * `indices` - Work vector n_blocks long
/// * `lengths` - Work vector n_blocks long
/// * `flags` - Work vector n_blocks long
/// * `work` - Work vector ncases long
/// * `is_crits` - Work vector n_systems long
/// * `oos_crits` - Work vector n_systems long
///
/// # Returns
/// Probability that IS best is at or below OOS median
pub fn cscvcore(
    ncases: usize,
    n_systems: usize,
    n_blocks_input: usize,
    returns: &[f64],
    indices: &mut Vec<usize>,
    lengths: &mut Vec<usize>,
    flags: &mut Vec<u8>,
    work: &mut Vec<f64>,
    is_crits: &mut Vec<f64>,
    oos_crits: &mut Vec<f64>,
) -> f64 {
    // Ensure n_blocks is even
    let n_blocks = (n_blocks_input / 2) * 2;

    // Validate input dimensions
    assert_eq!(
        returns.len(),
        n_systems * ncases,
        "Returns matrix size mismatch"
    );
    assert!(n_blocks > 0 && n_blocks % 2 == 0, "n_blocks must be even and > 0");

    // Ensure work vectors are properly sized
    indices.clear();
    indices.resize(n_blocks, 0);
    lengths.clear();
    lengths.resize(n_blocks, 0);
    flags.clear();
    flags.resize(n_blocks, 0);
    work.clear();
    work.resize(ncases, 0.0);
    is_crits.clear();
    is_crits.resize(n_systems, 0.0);
    oos_crits.clear();
    oos_crits.resize(n_systems, 0.0);

    /*
       Find the starting index and length of each of the n_blocks submatrices.
       Ideally, ncases should be an integer multiple of n_blocks so that
       all submatrices are the same size.
    */

    let mut istart = 0;
    for i in 0..n_blocks {
        indices[i] = istart;
        lengths[i] = (ncases - istart) / (n_blocks - i);
        istart += lengths[i];
    }

    /*
       Initialize
    */

    let mut nless = 0; // Will count the number of time OOS of best <= median OOS

    // Identify the training set blocks
    for i in 0..(n_blocks / 2) {
        flags[i] = 1;
    }

    // Identify the test set blocks
    for i in (n_blocks / 2)..n_blocks {
        flags[i] = 0;
    }

    /*
       Main loop processes all combinations of blocks
    */

    let mut ncombo = 0;
    loop {
        /*
           Compute training-set (IS) criterion for each candidate system
        */

        for isys in 0..n_systems {
            // Each row of returns matrix
            let mut n = 0; // Counts cases in training set

            for ic in 0..n_blocks {
                // For all blocks (sub-matrices)
                if flags[ic] == 1 {
                    // If this block is in the training set
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        // For every case in this block
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }

            is_crits[isys] = criter(&work[0..n]);
        }

        /*
           Compute (OOS) criterion for each candidate system
        */

        for isys in 0..n_systems {
            // Each column of returns matrix
            let mut n = 0; // Counts cases in OOS set

            for ic in 0..n_blocks {
                // For all blocks (sub-matrices)
                if flags[ic] == 0 {
                    // If this block is in the OOS set
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        // For every case in this block
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }

            oos_crits[isys] = criter(&work[0..n]);
        }

        /*
           Determine the relative rank within OOS of the system which had best IS performance.
        */

        // Find the best system IS
        let mut best = f64::NEG_INFINITY;
        let mut ibest = 0;
        for isys in 0..n_systems {
            if is_crits[isys] > best {
                best = is_crits[isys];
                ibest = isys;
            }
        }

        best = oos_crits[ibest]; // This is the OOS value for the best system IS
        let mut n = 0;
        for isys in 0..n_systems {
            if isys == ibest || best >= oos_crits[isys] {
                // Insurance against floating point error
                n += 1;
            }
        }

        let rel_rank = n as f64 / (n_systems as f64 + 1.0);
        // logit = log(rel_rank / (1.0 - rel_rank)); // Optional

        if rel_rank <= 0.5 {
            // Is the IS best at or below the OOS median?
            nless += 1;
        }

        /*
           Move to the next combination
        */

        let mut found_next = false;
        let mut n = 0;
        for iradix in 0..(n_blocks - 1) {
            if flags[iradix] == 1 {
                n += 1; // This many flags up to and including this one at iradix
                if flags[iradix + 1] == 0 {
                    flags[iradix] = 0;
                    flags[iradix + 1] = 1;

                    // Must reset everything below this change point
                    for i in 0..iradix {
                        n -= 1;
                        if n > 0 {
                            flags[i] = 1;
                        } else {
                            flags[i] = 0;
                        }
                    }

                    found_next = true;
                    break;
                }
            }
        }

        ncombo += 1;

        if !found_next {
            // Last combination reached
            break;
        }
    }

    nless as f64 / ncombo as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criter_basic() {
        let returns = vec![1.0, 2.0, 3.0, 4.0];
        let result = criter(&returns);
        assert!((result - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_criter_empty() {
        let returns: Vec<f64> = vec![];
        let result = criter(&returns);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_criter_single() {
        let returns = vec![5.0];
        let result = criter(&returns);
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cscvcore_small() {
        let ncases = 12;
        let n_systems = 4;
        let n_blocks = 4;

        // Create simple test data
        let returns = vec![0.1; n_systems * ncases];

        let mut indices = Vec::new();
        let mut lengths = Vec::new();
        let mut flags = Vec::new();
        let mut work = Vec::new();
        let mut is_crits = Vec::new();
        let mut oos_crits = Vec::new();

        let prob = cscvcore(
            ncases,
            n_systems,
            n_blocks,
            &returns,
            &mut indices,
            &mut lengths,
            &mut flags,
            &mut work,
            &mut is_crits,
            &mut oos_crits,
        );

        assert!(prob >= 0.0 && prob <= 1.0, "Probability should be in [0, 1]");
    }

    #[test]
    fn test_cscvcore_block_distribution() {
        let ncases = 100;
        let n_systems = 3;
        let n_blocks = 4;

        let returns = vec![0.05; n_systems * ncases];

        let mut indices = Vec::new();
        let mut lengths = Vec::new();
        let mut flags = Vec::new();
        let mut work = Vec::new();
        let mut is_crits = Vec::new();
        let mut oos_crits = Vec::new();

        cscvcore(
            ncases,
            n_systems,
            n_blocks,
            &returns,
            &mut indices,
            &mut lengths,
            &mut flags,
            &mut work,
            &mut is_crits,
            &mut oos_crits,
        );

        // Verify block structure
        assert_eq!(indices.len(), n_blocks);
        assert_eq!(lengths.len(), n_blocks);

        // Check that blocks cover all cases
        let total_cases: usize = lengths.iter().sum();
        assert_eq!(total_cases, ncases);
    }

    #[test]
    fn test_cscvcore_even_blocks() {
        let ncases = 20;
        let n_systems = 2;
        let n_blocks = 5; // Will be converted to 4

        let returns = vec![0.1; n_systems * ncases];

        let mut indices = Vec::new();
        let mut lengths = Vec::new();
        let mut flags = Vec::new();
        let mut work = Vec::new();
        let mut is_crits = Vec::new();
        let mut oos_crits = Vec::new();

        let prob = cscvcore(
            ncases,
            n_systems,
            n_blocks,
            &returns,
            &mut indices,
            &mut lengths,
            &mut flags,
            &mut work,
            &mut is_crits,
            &mut oos_crits,
        );

        // Verify n_blocks was made even
        assert_eq!(indices.len(), 4);
        assert!(prob >= 0.0 && prob <= 1.0);
    }

    #[test]
    fn test_cscvcore_varying_returns() {
        let ncases = 16;
        let n_systems = 3;
        let n_blocks = 4;

        // Create returns with variation
        let mut returns = vec![0.0; n_systems * ncases];
        for i in 0..returns.len() {
            returns[i] = (i as f64) * 0.01;
        }

        let mut indices = Vec::new();
        let mut lengths = Vec::new();
        let mut flags = Vec::new();
        let mut work = Vec::new();
        let mut is_crits = Vec::new();
        let mut oos_crits = Vec::new();

        let prob = cscvcore(
            ncases,
            n_systems,
            n_blocks,
            &returns,
            &mut indices,
            &mut lengths,
            &mut flags,
            &mut work,
            &mut is_crits,
            &mut oos_crits,
        );

        assert!(prob >= 0.0 && prob <= 1.0);
    }
}