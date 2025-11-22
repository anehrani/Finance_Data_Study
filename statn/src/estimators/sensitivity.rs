use std::fs::File;
use std::io::{self, Write};

/// Compute and print parameter sensitivity curves
///
/// This function evaluates how the criterion function varies as each parameter
/// is varied across its range while holding other parameters at their optimal values.
/// Results are written to SENS.LOG as ASCII histograms.
///
/// # Arguments
/// * `criter` - Criterion function to evaluate. Takes parameters and mintrades.
/// * `nvars` - Number of variables
/// * `nints` - Number of first variables that are integers
/// * `npoints` - Number of points at which to evaluate performance
/// * `nres` - Number of resolved points across plot (histogram width)
/// * `mintrades` - Minimum number of trades
/// * `best` - Optimal parameters found
/// * `low_bounds` - Lower bounds for parameters
/// * `high_bounds` - Upper bounds for parameters
///
/// # Returns
/// `Ok(())` on success, or an IO error if file writing fails
pub fn sensitivity<F>(
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
        // Reset params to optimal values
        for i in 0..nvars {
            params[i] = best[i];
        }

        let mut maxval = -1.0e60;

        if ivar < nints {
            // Integer parameter
            writeln!(
                fp,
                "\n\nSensitivity curve for integer parameter {} (optimum={})",
                ivar + 1,
                (best[ivar] + 1.0e-10) as i32
            )?;

            let label_frac =
                (high_bounds[ivar] - low_bounds[ivar] + 0.99999999) / (npoints as f64 - 1.0);

            // Evaluate criterion at each point
            for ipoint in 0..npoints {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                params[ivar] = ival as f64;
                vals[ipoint] = criter(&params, mintrades);
                if ipoint == 0 || vals[ipoint] > maxval {
                    maxval = vals[ipoint];
                }
            }

            // Print histogram
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
            // Real parameter
            writeln!(
                fp,
                "\n\nSensitivity curve for real parameter {} (optimum={:.4})",
                ivar + 1,
                best[ivar]
            )?;

            let label_frac = (high_bounds[ivar] - low_bounds[ivar]) / (npoints as f64 - 1.0);

            // Evaluate criterion at each point
            for ipoint in 0..npoints {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                params[ivar] = rval;
                vals[ipoint] = criter(&params, mintrades);
                if ipoint == 0 || vals[ipoint] > maxval {
                    maxval = vals[ipoint];
                }
            }

            // Print histogram
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
