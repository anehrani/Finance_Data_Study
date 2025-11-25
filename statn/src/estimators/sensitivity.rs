use std::io;

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
    let mut buffer = String::new();
    let mut params = best.to_vec();
    let mut vals = vec![0.0; npoints];

    for ivar in 0..nvars {
        // Reset params to optimal values
        params[..nvars].copy_from_slice(&best[..nvars]);

        let mut maxval = -1.0e60;

        if ivar < nints {
            // Integer parameter
            use std::fmt::Write;
            writeln!(
                buffer,
                "\n\nSensitivity curve for integer parameter {} (optimum={})",
                ivar + 1,
                (best[ivar] + 1.0e-10) as i32
            ).unwrap();

            let label_frac =
                (high_bounds[ivar] - low_bounds[ivar] + 0.99999999) / (npoints as f64 - 1.0);

            // Evaluate criterion at each point
            for (ipoint, val) in vals.iter_mut().enumerate().take(npoints) {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                params[ivar] = ival as f64;
                *val = criter(&params, mintrades);
                if ipoint == 0 || *val > maxval {
                    maxval = *val;
                }
            }

            // Print histogram
            let hist_frac = (nres as f64 + 0.9999999) / maxval.abs().max(1.0e-9);

            for (ipoint, &val) in vals.iter().enumerate().take(npoints) {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                write!(buffer, "\n{:6}|", ival).unwrap();
                let k = (val * hist_frac) as i32;
                for _ in 0..k {
                    write!(buffer, "*").unwrap();
                }
            }
        } else {
            // Real parameter
            use std::fmt::Write;
            writeln!(
                buffer,
                "\n\nSensitivity curve for real parameter {} (optimum={:.4})",
                ivar + 1,
                best[ivar]
            ).unwrap();

            let label_frac = (high_bounds[ivar] - low_bounds[ivar]) / (npoints as f64 - 1.0);

            // Evaluate criterion at each point
            for (ipoint, val) in vals.iter_mut().enumerate().take(npoints) {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                params[ivar] = rval;
                *val = criter(&params, mintrades);
                if ipoint == 0 || *val > maxval {
                    maxval = *val;
                }
            }

            // Print histogram
            let hist_frac = (nres as f64 + 0.9999999) / maxval.abs().max(1.0e-9);

            for (ipoint, &val) in vals.iter().enumerate().take(npoints) {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                write!(buffer, "\n{:10.3}|", rval).unwrap();
                let k = (val * hist_frac) as i32;
                for _ in 0..k {
                    write!(buffer, "*").unwrap();
                }
            }
        }
    }

    crate::core::io::write::write_file("SENS.LOG", buffer)
}
