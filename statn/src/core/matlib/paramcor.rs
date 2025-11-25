

/// Compute and print parameter correlation information
///
/// This function analyzes the correlation between parameters by fitting
/// a quadratic model to the vicinity of the best solution and examining
/// the Hessian matrix and its eigenstructure.
///
/// # Arguments
/// * `data` - Ncases (rows) by nparams+1, where each row contains parameters
///            followed by the function value
/// * `nparams` - Number of parameters
///
/// # Returns
/// `Ok(())` on success, `Err(String)` on failure
pub fn paramcor(data: &[f64], nparams: usize) -> Result<String, String> {
    let ncases = data.len() / (nparams + 1);

    if nparams < 2 {
        return Err("Need at least 2 parameters".to_string());
    }

    // Calculate number of coefficients for quadratic model:
    // First-order terms (nparams) + second-order terms (nparams*(nparams+1)/2) + constant (1)
    let ncoefs = nparams + nparams * (nparams + 1) / 2 + 1;

    // Keep the closest individuals to the best (heuristic: 1.5 * ncoefs)
    let mut nc_kept = (1.5 * ncoefs as f64) as usize;
    if nc_kept > ncases {
        nc_kept = ncases;
    }

    // Open log file buffer
    let mut buffer = String::new();

    // Find the best individual (maximum function value)
    let mut ibest = 0;
    let mut best_val = f64::NEG_INFINITY;

    for i in 0..ncases {
        let fval = data[i * (nparams + 1) + nparams];
        if fval > best_val {
            ibest = i;
            best_val = fval;
        }
    }

    let best = &data[ibest * (nparams + 1)..(ibest + 1) * (nparams + 1)];

    // Compute distances from best individual
    let mut distances: Vec<f64> = vec![0.0; ncases];
    let mut indices: Vec<usize> = (0..ncases).collect();

    for i in 0..ncases {
        let ind = i * (nparams + 1);
        let mut sum = 0.0;
        for j in 0..nparams {
            let diff = data[ind + j] - best[j];
            sum += diff * diff;
        }
        distances[i] = sum;
    }

    // Sort by distance
    indices.sort_by(|&a, &b| {
        distances[a]
            .partial_cmp(&distances[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Build design matrix and RHS
    let mut a_matrix = vec![0.0; nc_kept * ncoefs];
    let mut b_vector = vec![0.0; nc_kept];

    for i in 0..nc_kept {
        let case_idx = indices[i];
        let pptr = &data[case_idx * (nparams + 1)..(case_idx + 1) * (nparams + 1)];

        let mut row_idx = i * ncoefs;

        // First-order terms and second-order terms
        for j in 0..nparams {
            let d = pptr[j] - best[j];
            a_matrix[row_idx] = d;
            row_idx += 1;

            for k in j..nparams {
                let d2 = pptr[k] - best[k];
                a_matrix[row_idx] = d * d2;
                row_idx += 1;
            }
        }

        // Constant term
        a_matrix[row_idx] = 1.0;

        // RHS: flip sign to convert maximum to minimum
        b_vector[i] = best[nparams] - pptr[nparams];
    }

    // Perform SVD and back-substitution to get coefficients
    let mut coefs = vec![0.0; ncoefs];
    let svd = svd_solve(&a_matrix, &b_vector, nc_kept, ncoefs)?;
    coefs.copy_from_slice(&svd);

    // Print coefficients
    use std::fmt::Write;
    writeln!(
        buffer,
        "Coefficients fitting performance to parameters, linear first, then quadratic, then mixed"
    )
    .map_err(|e| format!("Write error: {}", e))?;

    let mut coef_idx = 0;
    for j in 0..nparams {
        let mut line = format!("{:11.3e} :", coefs[coef_idx]);
        coef_idx += 1;

        for k in j..nparams {
            line.push_str(&format!(" {:11.3e}", coefs[coef_idx]));
            coef_idx += 1;
        }
        writeln!(buffer, "{}", line).map_err(|e| format!("Write error: {}", e))?;
    }
    writeln!(buffer, "Constant: {:.3e}", coefs[coef_idx])
        .map_err(|e| format!("Write error: {}", e))?;

    // Build Hessian matrix
    let mut hessian = vec![0.0; nparams * nparams];
    coef_idx = nparams; // Skip linear terms

    for j in 0..nparams {
        for k in j..nparams {
            let mut val = coefs[coef_idx];
            if k == j {
                val *= 2.0; // Diagonal: second partial is twice the coefficient
            }
            hessian[j * nparams + k] = val;
            hessian[k * nparams + j] = val;
            coef_idx += 1;
        }
    }

    // Print original Hessian
    writeln!(buffer, "\n\nHessian before adjustment").map_err(|e| format!("Write error: {}", e))?;
    print_matrix(&mut buffer, &hessian, nparams)?;

    // Adjust Hessian: zero out non-positive diagonals and their rows/columns
    for j in 0..nparams {
        if hessian[j * nparams + j] < 1e-10 {
            for k in 0..nparams {
                hessian[j * nparams + k] = 0.0;
                hessian[k * nparams + j] = 0.0;
            }
        }
    }

    // Enforce positive semi-definiteness constraints
    for j in 0..nparams - 1 {
        let d = hessian[j * nparams + j];
        for k in (j + 1)..nparams {
            let d2 = hessian[k * nparams + k];
            let limit = 0.99999 * (d * d2).sqrt();

            if hessian[j * nparams + k] > limit {
                hessian[j * nparams + k] = limit;
                hessian[k * nparams + j] = limit;
            }
            if hessian[j * nparams + k] < -limit {
                hessian[j * nparams + k] = -limit;
                hessian[k * nparams + j] = -limit;
            }
        }
    }

    // Print adjusted Hessian
    writeln!(
        buffer,
        "\n\nHessian after adjustment to encourage nonnegative eigenvalues"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    print_matrix(&mut buffer, &hessian, nparams)?;

    // Compute eigenstructure
    let (evals, evect) = eigen_decomposition(&hessian, nparams)?;

    // Print eigenvalues and eigenvectors
    writeln!(
        buffer,
        "\n\nEigenvalues (top row) with corresponding vectors below each"
    )
    .map_err(|e| format!("Write error: {}", e))?;

    let mut line = String::new();
    for j in 0..nparams {
        line.push_str(&format!(" {:11.3e}", evals[j]));
    }
    writeln!(buffer, "{}", line).map_err(|e| format!("Write error: {}", e))?;

    for j in 0..nparams {
        let mut line = String::new();
        for k in 0..nparams {
            line.push_str(&format!(" {:11.3e}", evect[j * nparams + k]));
        }
        writeln!(buffer, "{}", line).map_err(|e| format!("Write error: {}", e))?;
    }

    // Compute generalized inverse of Hessian
    let mut hessian_inv = vec![0.0; nparams * nparams];
    for j in 0..nparams {
        for k in j..nparams {
            let mut sum = 0.0;
            for i in 0..nparams {
                if evals[i] > 1e-8 {
                    sum += evect[j * nparams + i] * evect[k * nparams + i] / evals[i];
                }
            }
            hessian_inv[j * nparams + k] = sum;
            hessian_inv[k * nparams + j] = sum;
        }
    }

    writeln!(buffer, "\n\nGeneralized inverse of modified Hessian")
        .map_err(|e| format!("Write error: {}", e))?;
    print_matrix(&mut buffer, &hessian_inv, nparams)?;

    // Print parameter variation and correlations
    writeln!(
        buffer,
        "\n\nEstimated parameter variation and correlations\n"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "Variation very roughly indicates how much the parameter can change"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "RELATIVE to the others without having a huge impact on performance.\n"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "A strong positive correlation between A and B means that an increase"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "in parameter A can be somewhat offset by an increase in parameter B.\n"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "A strong negative correlation between A and B means that an increase"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        buffer,
        "in parameter A can be somewhat offset by a decrease in parameter B.\n"
    )
    .map_err(|e| format!("Write error: {}", e))?;

    // Find maximum variation for scaling
    let mut rscale = 0.0;
    for i in 0..nparams {
        if hessian_inv[i * nparams + i] > 0.0 {
            let d = hessian_inv[i * nparams + i].sqrt();
            if d > rscale {
                rscale = d;
            }
        }
    }

    // Print variations
    let mut header = "               ".to_string();
    for i in 0..nparams {
        header.push_str(&format!("      Param {}", i + 1));
    }
    writeln!(buffer, "{}", header).map_err(|e| format!("Write error: {}", e))?;

    let mut var_line = "  Variation-->".to_string();
    for i in 0..nparams {
        let d = if hessian_inv[i * nparams + i] > 0.0 {
            hessian_inv[i * nparams + i].sqrt() / rscale
        } else {
            0.0
        };
        var_line.push_str(&format!(" {:12.3}", d));
    }
    writeln!(buffer, "{}", var_line).map_err(|e| format!("Write error: {}", e))?;

    // Print correlations
    for i in 0..nparams {
        let mut line = format!("  {:12}", i + 1);
        let d = if hessian_inv[i * nparams + i] > 0.0 {
            hessian_inv[i * nparams + i].sqrt()
        } else {
            0.0
        };

        for k in 0..nparams {
            let d2 = if hessian_inv[k * nparams + k] > 0.0 {
                hessian_inv[k * nparams + k].sqrt()
            } else {
                0.0
            };

            if d * d2 > 0.0 {
                let mut corr = hessian_inv[i * nparams + k] / (d * d2);
                corr = corr.max(-1.0).min(1.0);
                line.push_str(&format!(" {:12.3}", corr));
            } else {
                line.push_str("        -----");
            }
        }
        writeln!(buffer, "{}", line).map_err(|e| format!("Write error: {}", e))?;
    }

    // Print sensitivity vectors if applicable
    if nparams >= 2 {
        // Find smallest positive eigenvalue
        let mut min_pos_idx = None;
        for k in (0..nparams).rev() {
            if evals[k] > 0.0 {
                min_pos_idx = Some(k);
                break;
            }
        }

        if let Some(k) = min_pos_idx {
            writeln!(
                buffer,
                "\n\nDirections of maximum and minimum sensitivity"
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(
                buffer,
                "Moving in the direction of maximum sensitivity causes the most change in performance."
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(
                buffer,
                "Moving in the direction of minimum sensitivity causes the least change in performance.\n"
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(buffer, "                     Max        Min\n")
                .map_err(|e| format!("Write error: {}", e))?;

            let mut lscale = 0.0;
            let mut rscale_local = 0.0;

            for i in 0..nparams {
                if evect[i * nparams].abs() > lscale {
                    lscale = evect[i * nparams].abs();
                }
                if evect[i * nparams + k].abs() > rscale_local {
                    rscale_local = evect[i * nparams + k].abs();
                }
            }

            for i in 0..nparams {
                let max_val = if lscale > 0.0 {
                    evect[i * nparams] / lscale
                } else {
                    0.0
                };
                let min_val = if rscale_local > 0.0 {
                    evect[i * nparams + k] / rscale_local
                } else {
                    0.0
                };
                writeln!(
                    buffer,
                    "       Param {} {:10.3} {:10.3}",
                    i + 1,
                    max_val,
                    min_val
                )
                .map_err(|e| format!("Write error: {}", e))?;
            }
        }
    }

    Ok(buffer)
}

/// Helper function to print a matrix
/// Helper function to print a matrix
pub fn print_matrix(
    buffer: &mut String,
    matrix: &[f64],
    n: usize,
) -> Result<(), String> {
    use std::fmt::Write;
    for j in 0..n {
        let mut line = String::new();
        for k in 0..n {
            line.push_str(&format!(" {:11.3e}", matrix[j * n + k]));
        }
        writeln!(buffer, "{}", line).map_err(|e| format!("Write error: {}", e))?;
    }
    Ok(())
}

/// Simple SVD solver using QR decomposition (simplified implementation)
pub fn svd_solve(
    a_matrix: &[f64],
    b_vector: &[f64],
    nrows: usize,
    ncols: usize,
) -> Result<Vec<f64>, String> {
    // Use a normal equations approach: (A'A)x = A'b
    let mut ata = vec![0.0; ncols * ncols];
    let mut atb = vec![0.0; ncols];

    // Compute A'A
    for i in 0..ncols {
        for j in 0..ncols {
            let mut sum = 0.0;
            for k in 0..nrows {
                sum += a_matrix[k * ncols + i] * a_matrix[k * ncols + j];
            }
            ata[i * ncols + j] = sum;
        }
    }

    // Compute A'b
    for i in 0..ncols {
        let mut sum = 0.0;
        for k in 0..nrows {
            sum += a_matrix[k * ncols + i] * b_vector[k];
        }
        atb[i] = sum;
    }

    // Solve using Gaussian elimination with partial pivoting
    gauss_elimination(&ata, &atb, ncols)
}

/// Simple Gaussian elimination solver
pub fn gauss_elimination(a: &[f64], b: &[f64], n: usize) -> Result<Vec<f64>, String> {
    let mut a = a.to_vec();
    let mut b = b.to_vec();

    // Forward elimination
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        for row in (col + 1)..n {
            if a[row * n + col].abs() > a[max_row * n + col].abs() {
                max_row = row;
            }
        }

        // Swap rows
        if max_row != col {
            for j in 0..n {
                a.swap(col * n + j, max_row * n + j);
            }
            b.swap(col, max_row);
        }

        if a[col * n + col].abs() < 1e-15 {
            return Err("Matrix is singular".to_string());
        }

        // Eliminate below
        for row in (col + 1)..n {
            let factor = a[row * n + col] / a[col * n + col];
            for j in col..n {
                a[row * n + j] -= factor * a[col * n + j];
            }
            b[row] -= factor * b[col];
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        x[i] = b[i];
        for j in (i + 1)..n {
            x[i] -= a[i * n + j] * x[j];
        }
        x[i] /= a[i * n + i];
    }

    Ok(x)
}

/// Compute eigenvalues and eigenvectors using power iteration
/// This is a simplified implementation; for production use, consider using the `ndarray` crate
pub fn eigen_decomposition(
    matrix: &[f64],
    n: usize,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    // For a symmetric matrix, use the Jacobi eigenvalue algorithm
    let mut a = matrix.to_vec();
    let mut v = vec![0.0; n * n];

    // Initialize v to identity matrix
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    // Jacobi iterations
    for _ in 0..100 {
        let mut max_off_diag = 0.0;
        let mut max_i = 0;
        let mut max_j = 1;

        // Find largest off-diagonal element
        for i in 0..n {
            for j in (i + 1)..n {
                if a[i * n + j].abs() > max_off_diag {
                    max_off_diag = a[i * n + j].abs();
                    max_i = i;
                    max_j = j;
                }
            }
        }

        if max_off_diag < 1e-12 {
            break;
        }

        // Compute Givens rotation
        let aii = a[max_i * n + max_i];
        let ajj = a[max_j * n + max_j];
        let aij = a[max_i * n + max_j];

        let tau = (ajj - aii) / (2.0 * aij);
        let t = if tau >= 0.0 {
            1.0 / (tau + (tau * tau + 1.0).sqrt())
        } else {
            1.0 / (tau - (tau * tau + 1.0).sqrt())
        };

        let c = 1.0 / (t * t + 1.0).sqrt();
        let s = t * c;

        // Update matrix
        for k in 0..n {
            if k != max_i && k != max_j {
                let aik = a[max_i * n + k];
                let ajk = a[max_j * n + k];
                a[max_i * n + k] = c * aik - s * ajk;
                a[k * n + max_i] = a[max_i * n + k];
                a[max_j * n + k] = s * aik + c * ajk;
                a[k * n + max_j] = a[max_j * n + k];
            }
        }

        let aii_new = c * c * aii + s * s * ajj - 2.0 * s * c * aij;
        let ajj_new = s * s * aii + c * c * ajj + 2.0 * s * c * aij;
        a[max_i * n + max_i] = aii_new;
        a[max_j * n + max_j] = ajj_new;
        a[max_i * n + max_j] = 0.0;
        a[max_j * n + max_i] = 0.0;

        // Update eigenvectors
        for k in 0..n {
            let vik = v[k * n + max_i];
            let vjk = v[k * n + max_j];
            v[k * n + max_i] = c * vik - s * vjk;
            v[k * n + max_j] = s * vik + c * vjk;
        }
    }

    // Extract eigenvalues
    let mut evals = vec![0.0; n];
    for i in 0..n {
        evals[i] = a[i * n + i];
    }

    Ok((evals, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_elimination() {
        // Test: solve 2x + 3y = 8, 4x + y = 10
        // Solution: x = 1, y = 2
        let a = vec![2.0, 3.0, 4.0, 1.0];
        let b = vec![8.0, 10.0];

        let result = gauss_elimination(&a, &b, 2);
        assert!(result.is_ok());
        let x = result.unwrap();
        assert!((x[0] - 1.0).abs() < 1e-10);
        assert!((x[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_paramcor_simple() {
        // Create simple test data
        // 3 cases, 2 parameters each
        let data = vec![
            1.0, 2.0, 10.0, // params: 1, 2; fval: 10 (best)
            1.1, 2.1, 9.5,  // params: 1.1, 2.1; fval: 9.5
            0.9, 1.9, 8.0,  // params: 0.9, 1.9; fval: 8.0
        ];

        let result = paramcor(&data, 2);
        assert!(result.is_ok());
        assert!(std::path::Path::new("PARAMCOR.LOG").exists());
    }
}

// fn main() {
//     println!("Parameter Correlation Analysis Tool\n");

//     // Example data: 10 cases, 3 parameters each
//     let mut data = vec![];

//     // Best case
//     data.extend_from_slice(&[5.0, 3.0, 2.0, 100.0]);

//     // Nearby cases with slightly worse values
//     for i in 0..9 {
//         let p1 = 5.0 + (i as f64 - 4.0) * 0.1;
//         let p2 = 3.0 + (i as f64 - 4.0) * 0.15;
//         let p3 = 2.0 + (i as f64 - 4.0) * 0.05;
//         let fval = 100.0 - (i as f64 + 1.0) * 2.0;
//         data.extend_from_slice(&[p1, p2, p3, fval]);
//     }

//     println!("Running parameter correlation analysis...");
//     println!("  Data points: {}", data.len() / 4);
//     println!("  Parameters: 3\n");

//     match paramcor(&data, 3) {
//         Ok(()) => {
//             println!("✓ Analysis completed successfully");
//             println!("✓ Results written to PARAMCOR.LOG\n");

//             // Display log file contents
//             match std::fs::read_to_string("PARAMCOR.LOG") {
//                 Ok(content) => {
//                     println!("--- PARAMCOR.LOG Contents ---");
//                     println!("{}", content);
//                 }
//                 Err(e) => println!("Could not read PARAMCOR.LOG: {}", e),
//             }
//         }
//         Err(e) => {
//             println!("✗ Error during analysis: {}", e);
//         }
//     }
// }