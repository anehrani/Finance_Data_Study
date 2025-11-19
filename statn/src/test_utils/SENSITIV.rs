use std::fs::File;
use std::io::Write;

/// Compute and optionally print parameter sensitivity curves
///
/// This function evaluates how the criterion function responds to changes
/// in each parameter while holding others constant at their optimal values.
///
/// # Arguments
/// * `criter` - Criterion function to be maximized, takes parameters and min_trades
/// * `nvars` - Number of variables/parameters
/// * `nints` - Number of first variables that are integers (rest are real)
/// * `npoints` - Number of points at which to evaluate performance
/// * `nres` - Number of resolved points across plot (max histogram width)
/// * `min_trades` - Minimum number of trades
/// * `best` - Optimal parameter values
/// * `low_bounds` - Lower bounds for each parameter
/// * `high_bounds` - Upper bounds for each parameter
///
/// # Returns
/// `Ok(())` on success, `Err(String)` on failure
pub fn sensitivity(
    criter: &dyn Fn(&[f64], usize) -> f64,
    nvars: usize,
    nints: usize,
    npoints: usize,
    nres: usize,
    min_trades: usize,
    best: &[f64],
    low_bounds: &[f64],
    high_bounds: &[f64],
) -> Result<(), String> {
    // Validate input dimensions
    if best.len() != nvars
        || low_bounds.len() != nvars
        || high_bounds.len() != nvars
    {
        return Err("Input array dimensions mismatch".to_string());
    }

    if nints > nvars {
        return Err("Number of integers cannot exceed number of variables".to_string());
    }

    if npoints < 2 {
        return Err("npoints must be at least 2".to_string());
    }

    // Open the log file for writing
    let mut fp = File::create("SENS.LOG")
        .map_err(|e| format!("Failed to create SENS.LOG: {}", e))?;

    // Process each variable
    for ivar in 0..nvars {
        let mut params = best.to_vec();

        // Process integer parameters
        if ivar < nints {
            let optimum = (best[ivar] + 1e-10) as i32;
            writeln!(
                fp,
                "\n\nSensitivity curve for integer parameter {} (optimum={})",
                ivar + 1,
                optimum
            )
            .map_err(|e| format!("Write error: {}", e))?;

            // Calculate spacing for integer parameter values
            let range = high_bounds[ivar] - low_bounds[ivar];
            let label_frac = (range + 0.99999999) / (npoints - 1) as f64;

            // Evaluate criterion at each point
            let mut vals = Vec::with_capacity(npoints);
            let mut maxval = f64::NEG_INFINITY;

            for ipoint in 0..npoints {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                params[ivar] = ival as f64;
                let val = criter(&params, min_trades);
                vals.push(val);
                if val > maxval {
                    maxval = val;
                }
            }

            // Print histogram
            let hist_frac = (nres as f64 + 0.9999999) / maxval;
            for ipoint in 0..npoints {
                let ival = (low_bounds[ivar] + ipoint as f64 * label_frac) as i32;
                let k = (vals[ipoint] * hist_frac) as usize;
                
                write!(fp, "\n{:6}|", ival)
                    .map_err(|e| format!("Write error: {}", e))?;
                
                for _ in 0..k {
                    write!(fp, "*").map_err(|e| format!("Write error: {}", e))?;
                }
            }
        }
        // Process real parameters
        else {
            writeln!(
                fp,
                "\n\nSensitivity curve for real parameter {} (optimum={:.4})",
                ivar + 1,
                best[ivar]
            )
            .map_err(|e| format!("Write error: {}", e))?;

            // Calculate spacing for real parameter values
            let range = high_bounds[ivar] - low_bounds[ivar];
            let label_frac = range / (npoints - 1) as f64;

            // Evaluate criterion at each point
            let mut vals = Vec::with_capacity(npoints);
            let mut maxval = f64::NEG_INFINITY;

            for ipoint in 0..npoints {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                params[ivar] = rval;
                let val = criter(&params, min_trades);
                vals.push(val);
                if val > maxval {
                    maxval = val;
                }
            }

            // Print histogram
            let hist_frac = (nres as f64 + 0.9999999) / maxval;
            for ipoint in 0..npoints {
                let rval = low_bounds[ivar] + ipoint as f64 * label_frac;
                let k = (vals[ipoint] * hist_frac) as usize;
                
                write!(fp, "\n{:10.3}|", rval)
                    .map_err(|e| format!("Write error: {}", e))?;
                
                for _ in 0..k {
                    write!(fp, "*").map_err(|e| format!("Write error: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock criterion function for testing
    fn mock_criter(params: &[f64], _min_trades: usize) -> f64 {
        // Simple quadratic with maximum at params[0] = 5.0
        let diff = params[0] - 5.0;
        10.0 - diff * diff
    }

    #[test]
    fn test_basic_sensitivity() {
        let nvars = 1;
        let nints = 0;
        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0];

        let result = sensitivity(
            &mock_criter,
            nvars,
            nints,
            10, // npoints
            40, // nres
            1,  // min_trades
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_ok());
        
        // Check that the log file was created
        assert!(std::path::Path::new("SENS.LOG").exists());
    }

    #[test]
    fn test_dimension_mismatch() {
        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0, 20.0]; // Wrong dimension

        let result = sensitivity(
            &mock_criter,
            1,
            0,
            10,
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dimension"));
    }

    #[test]
    fn test_invalid_nints() {
        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0];

        let result = sensitivity(
            &mock_criter,
            1,
            5, // nints > nvars
            10,
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_points() {
        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0];

        let result = sensitivity(
            &mock_criter,
            1,
            0,
            1, // npoints < 2
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_variables() {
        fn multi_criter(params: &[f64], _min_trades: usize) -> f64 {
            // Different criterion for each parameter
            let mut result = 0.0;
            for (i, &p) in params.iter().enumerate() {
                let diff = p - (i as f64 + 1.0);
                result += 10.0 - diff * diff;
            }
            result
        }

        let nvars = 3;
        let nints = 1; // First parameter is integer
        let best = vec![1.0, 2.0, 3.0];
        let low_bounds = vec![0.0, 0.0, 0.0];
        let high_bounds = vec![5.0, 5.0, 5.0];

        let result = sensitivity(
            &multi_criter,
            nvars,
            nints,
            8,
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_ok());
        assert!(std::path::Path::new("SENS.LOG").exists());
    }

    #[test]
    fn test_integer_parameter_sensitivity() {
        fn int_criter(params: &[f64], _min_trades: usize) -> f64 {
            // Criterion that depends on integer parameter
            let int_val = params[0] as i32;
            (10 - (int_val - 5).abs()) as f64
        }

        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0];

        let result = sensitivity(
            &int_criter,
            1,
            1, // This is an integer parameter
            11,
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_output_file_format() {
        let best = vec![5.0];
        let low_bounds = vec![0.0];
        let high_bounds = vec![10.0];

        let _ = sensitivity(
            &mock_criter,
            1,
            0,
            5,
            40,
            1,
            &best,
            &low_bounds,
            &high_bounds,
        );

        // Verify file exists and has content
        let content = std::fs::read_to_string("SENS.LOG")
            .expect("Failed to read SENS.LOG");
        
        assert!(content.contains("Sensitivity curve"));
        assert!(content.contains("optimum"));
        assert!(content.contains("|"));
        assert!(content.contains("*"));
    }
}

fn main() {
    println!("Parameter Sensitivity Analysis Tool\n");

    // Example criterion function: quadratic with maximum at (5.0, 3.0)
    fn example_criter(params: &[f64], _min_trades: usize) -> f64 {
        if params.len() < 2 {
            return 0.0;
        }
        let diff1 = params[0] - 5.0;
        let diff2 = params[1] - 3.0;
        100.0 - 10.0 * diff1 * diff1 - 5.0 * diff2 * diff2
    }

    let nvars = 2;
    let nints = 1; // First parameter is integer
    let best = vec![5.0, 3.0];
    let low_bounds = vec![0.0, 0.0];
    let high_bounds = vec![10.0, 10.0];

    println!("Running sensitivity analysis...");
    println!("  Number of variables: {}", nvars);
    println!("  Number of integer variables: {}", nints);
    println!("  Optimal parameters: {:?}", best);
    println!("  Bounds: [{:?}, {:?}]", low_bounds, high_bounds);
    println!();

    match sensitivity(
        &example_criter,
        nvars,
        nints,
        15,  // npoints
        50,  // nres (histogram width)
        10,  // min_trades
        &best,
        &low_bounds,
        &high_bounds,
    ) {
        Ok(()) => {
            println!("✓ Sensitivity analysis completed successfully");
            println!("✓ Results written to SENS.LOG");
            
            // Display the contents of the log file
            match std::fs::read_to_string("SENS.LOG") {
                Ok(content) => {
                    println!("\n--- SENS.LOG Contents ---");
                    println!("{}", content);
                }
                Err(e) => println!("Could not read SENS.LOG: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Error during sensitivity analysis: {}", e);
        }
    }
}