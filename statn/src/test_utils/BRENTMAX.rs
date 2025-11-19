/******************************************************************************/
/*                                                                            */
/*  BRENTMAX - Use Brent's method to find a local maximum of a                */
/*             univariate function.                                           */
/*                                                                            */
/*  This is given three points such that the center has greater function      */
/*  value than its neighbors.  It iteratively refines the interval.           */
/*                                                                            */
/******************************************************************************/

const DEBUG: bool = false;

/// Use Brent's method to find a local maximum of a univariate function.
///
/// This is given three points such that the center has greater function
/// value than its neighbors. It iteratively refines the interval.
///
/// # Arguments
///
/// * `itmax` - Iteration limit
/// * `eps` - Function convergence tolerance
/// * `tol` - X convergence tolerance
/// * `c_func` - Criterion function
/// * `xa` - Lower X value, input and output
/// * `xb` - Middle (best), input and output
/// * `xc` - And upper, input and output
/// * `y` - Function value at xb
///
/// # Returns
///
/// The maximum function value found
pub fn brentmax<F>(
    itmax: usize,
    eps: f64,
    tol: f64,
    c_func: F,
    xa: &mut f64,
    xb: &mut f64,
    xc: &mut f64,
    y: f64,
) -> f64
where
    F: Fn(f64) -> f64,
{
    /*
       Initialize
    */
    let mut x0 = *xb;
    let mut x1 = *xb;
    let mut x2 = *xb;
    let mut xleft = *xa;
    let mut xright = *xc;

    let mut y0 = y;
    let mut y1 = y;
    let mut y2 = y;

    /*
      We want a golden-section search the first iteration.  Force this by setting
      movement equal to zero.
    */
    let mut movement = 0.0;
    let mut trial = 0.0;

    /*
       Main loop.
    */
    for _iter in 0..itmax {
        /*
        This test is more sophisticated than it looks.  It tests the closeness
        of xright and xleft (relative to small_dist), AND makes sure that x0 is
        near the midpoint of that interval.
        */

        let mut small_step = x0.abs();
        if small_step < 1.0 {
            small_step = 1.0;
        }
        small_step *= tol;
        let small_dist = 2.0 * small_step;

        let xmid = 0.5 * (xleft + xright);

        if (x0 - xmid).abs() <= (small_dist - 0.5 * (xright - xleft)) {
            break;
        }

        /*
           Avoid refining function to limits of precision
        */
        if (_iter >= 4) && (((y2 - y0).abs() / (y0.abs() + 1.0)) < eps) {
            break;
        }

        if movement.abs() > small_step {
            // Try parabolic only if moving
            if DEBUG {
                println!("\nTrying parabolic:");
            }

            let temp1 = (x0 - x2) * (y0 - y1);
            let temp2 = (x0 - x1) * (y0 - y2);
            let numer = (x0 - x1) * temp2 - (x0 - x2) * temp1;
            let denom = 2.0 * (temp1 - temp2);
            let testdist = movement; // Intervals must get smaller
            movement = trial;

            trial = if denom.abs() > 1.0e-40 {
                numer / denom // Parabolic estimate of maximum
            } else {
                1.0e40
            };

            let temp1_val = trial + x0;
            if (2.0 * trial.abs() < testdist.abs())
                && (temp1_val > xleft)
                && (temp1_val < xright)
            {
                // If shrinking and safely in bounds
                let this_x = temp1_val;
                if (this_x - xleft < small_dist) || (xright - this_x < small_dist) {
                    // Cannot get too close to the endpoints
                    trial = if x0 < xmid {
                        small_step
                    } else {
                        -small_step
                    };
                }
                if DEBUG {
                    println!(" GOOD");
                }
            } else {
                // Punt via golden section because cannot use parabolic
                movement = if xmid > x0 {
                    xright - x0
                } else {
                    xleft - x0
                };
                trial = 0.3819660 * movement;
                if DEBUG {
                    println!(" POOR");
                }
            }
        } else {
            // Must use golden section due to insufficient movement
            if DEBUG {
                println!("\nTrying golden.");
            }
            movement = if xmid > x0 {
                xright - x0
            } else {
                xleft - x0
            };
            trial = 0.3819660 * movement;
        }

        let this_x = if trial.abs() >= small_step {
            // Make sure we move a good distance
            x0 + trial
        } else if trial > 0.0 {
            x0 + small_step
        } else {
            x0 - small_step
        };

        /*
           Evaluate the function here.
        */
        let this_y = c_func(this_x);
        if DEBUG {
            println!(" Eval at {} = {}", this_x, this_y);
        }

        /*
           Insert this new point in the correct position in the 'best' hierarchy
        */
        if this_y >= y0 {
            // Improvement
            if this_x < x0 {
                xright = x0;
            } else {
                xleft = x0;
            }
            x2 = x1;
            x1 = x0;
            x0 = this_x;
            y2 = y1;
            y1 = y0;
            y0 = this_y;
        } else {
            // No improvement
            if this_x >= x0 {
                xright = this_x;
            } else {
                xleft = this_x;
            }

            if (this_y >= y1) || (x1 == x0) {
                x2 = x1;
                x1 = this_x;
                y2 = y1;
                y1 = this_y;
            } else if (this_y >= y2) || (x2 == x0) || (x2 == x1) {
                x2 = this_x;
                y2 = this_y;
            }
        }
    }

    *xa = xleft;
    *xb = x0;
    *xc = xright;

    y0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brentmax_simple_parabola() {
        // Test with a simple parabola: -(x-2)^2 + 5, maximum at x=2, y=5
        let parabola = |x: f64| -(x - 2.0).powi(2) + 5.0;

        let mut xa = 0.0;
        let mut xb = 2.0;
        let mut xc = 4.0;
        let y = parabola(xb);

        let max_y = brentmax(100, 1.0e-8, 0.0001, parabola, &mut xa, &mut xb, &mut xc, y);

        // Check that we found a maximum near x=2 with value near 5
        assert!((xb - 2.0).abs() < 0.001, "xb should be near 2.0, got {}", xb);
        assert!((max_y - 5.0).abs() < 0.001, "max_y should be near 5.0, got {}", max_y);
    }

    #[test]
    fn test_brentmax_cubic() {
        // Test with a cubic function that has a local maximum
        // f(x) = -x^3 + 3x^2, maximum at x=2, y=4
        let cubic = |x: f64| -x.powi(3) + 3.0 * x.powi(2);

        let mut xa = 1.0;
        let mut xb = 2.0;
        let mut xc = 3.0;
        let y = cubic(xb);

        let max_y = brentmax(100, 1.0e-8, 0.0001, cubic, &mut xa, &mut xb, &mut xc, y);

        // Check that we found the maximum near x=2 with value near 4
        assert!((xb - 2.0).abs() < 0.01, "xb should be near 2.0, got {}", xb);
        assert!((max_y - 4.0).abs() < 0.01, "max_y should be near 4.0, got {}", max_y);
    }
}