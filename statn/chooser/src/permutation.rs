use crate::random::Rng;

/// Prepare permutation by computing price changes
pub fn prepare_permute(
    nc: usize,
    nmkt: usize,
    offset: usize,
    data: &[Vec<f64>],
    changes: &mut [Vec<f64>],
) {
    for imarket in 0..nmkt {
        for icase in offset..nc {
            changes[imarket][icase] = data[imarket][icase] - data[imarket][icase - 1];
        }
    }
}

/// Perform permutation by shuffling changes and rebuilding prices
pub fn do_permute(
    nc: usize,
    nmkt: usize,
    offset: usize,
    data: &mut [Vec<f64>],
    changes: &mut [Vec<f64>],
    rng: &mut Rng,
) {
    // Shuffle the changes, permuting each market the same to preserve correlations
    let mut i = nc - offset;
    while i > 1 {
        let j = (rng.unifrand() * i as f64) as usize;
        let j = if j >= i { i - 1 } else { j };
        i -= 1;

        for imarket in 0..nmkt {
            changes[imarket].swap(i + offset, j + offset);
        }
    }

    // Rebuild the prices using the shuffled changes
    for imarket in 0..nmkt {
        for icase in offset..nc {
            data[imarket][icase] = data[imarket][icase - 1] + changes[imarket][icase];
        }
    }
}
