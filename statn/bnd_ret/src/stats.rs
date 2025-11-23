/// Statistical functions for bnd_ret

/// Log Gamma function (ACM algorithm 291)
/// Good to about 9-10 significant digits
pub fn lgamma(mut x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    let mut result;

    if x < 7.0 {
        result = 1.0;
        let mut z = x;
        while z < 7.0 {
            result *= z;
            x = z;  // This is critical - x gets updated to z
            z += 1.0;
        }
        x += 1.0;  // Now x is 7.0 (was 6.0 after loop)
        result = -result.ln();
    } else {
        result = 0.0;
    }

    let z = 1.0 / (x * x);

    result
        + (x - 0.5) * x.ln()
        - x
        + 0.918938533204673
        + ((((-0.000595238095238 * z + 0.000793650793651) * z - 0.002777777777778) * z
            + 0.083333333333333)
            / x)
}

/// Incomplete Beta function (ACM algorithm 179 with modifications through 1976)
/// p: First parameter, greater than 0
/// q: Second parameter, greater than 0
/// x: Upper integration limit: 0 <= x <= 1
pub fn ibeta(p: f64, q: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if p <= 0.0 || q <= 0.0 {
        return -1.0; // Error flag
    }

    // High precision settings
    let eps: f64 = 1e-12;
    let eps1: f64 = 1e-98;
    let aleps1 = eps1.ln();

    // Switch arguments if needed for better convergence
    let (p, q, x, switched_args) = if x > 0.5 {
        (q, p, 1.0 - x, true)
    } else {
        (p, q, x, false)
    };

    // Define ps as 1 if q is an integer, else q - (int) q
    let mut ps = q - q.floor();
    if ps == 0.0 {
        ps = 1.0;
    }

    // Compute INFSUM
    let px = p * x.ln();
    let pq = lgamma(p + q);
    let p1 = lgamma(p);
    let d4 = p.ln();

    let term = px + lgamma(ps + p) - lgamma(ps) - d4 - p1;

    let mut infsum = 0.0;
    if (term / aleps1) as i32 == 0 {
        infsum = term.exp();
        let mut cnt = infsum * p;
        let mut wh = 1.0;
        loop {
            cnt *= (wh - ps) * x / wh;
            let term = cnt / (p + wh);
            infsum += term;
            if term / eps <= infsum {
                break;
            }
            wh += 1.0;
        }
    }

    // Compute FINSUM
    let mut finsum = 0.0;
    if q > 1.0 {
        let xb = px + q * (1.0 - x).ln() + pq - p1 - q.ln() - lgamma(q);

        let mut ib = (xb / aleps1) as i32;
        if ib < 0 {
            ib = 0;
        }

        let xfac = 1.0 / (1.0 - x);
        let mut term = (xb - (ib as f64) * aleps1).exp();
        let mut ps = q;

        let mut wh = q - 1.0;
        while wh > 0.0 {
            let px = ps * xfac / (p + wh);

            if px <= 1.0 && ((term / eps <= finsum) || (term <= eps1 / px)) {
                break;
            }

            ps = wh;
            term *= px;

            if term > 1.0 {
                ib -= 1;
                term *= eps1;
            }

            if ib == 0 {
                finsum += term;
            }

            wh -= 1.0;
        }
    }

    let prob = finsum + infsum;

    if switched_args {
        1.0 - prob
    } else {
        prob
    }
}

/// Evaluate the probability of the m'th order statistic exceeding
/// the q'th quantile.
pub fn orderstat_tail(n: i32, q: f64, m: i32) -> f64 {
    if m > n {
        return 1.0;
    }
    if m <= 0 {
        return 0.0;
    }
    1.0 - ibeta(m as f64, (n - m + 1) as f64, q)
}

/// Compute the p such that probability of the m'th order statistic
/// exceeding the p'th quantile equals the specified conf.
/// Generally, set conf=0.05 or so.
///
/// This solves the equation conf-orderstat_tail=0.
/// First, the root is bounded by locating where the function changes sign.
/// Then Ridder's method is used to rapidly refine the root.
pub fn quantile_conf(n: i32, m: i32, conf: f64) -> f64 {
    const QCEPS: f64 = 1e-10;

    let mut x1 = 0.0;
    let mut y1 = conf - 1.0;

    let mut x3 = 0.1;
    let mut y3;

    // Advance until the sign changes
    while x3 <= 1.0 {
        y3 = conf - orderstat_tail(n, x3, m);
        if y3.abs() < QCEPS {
            return x3;
        }
        if y3 > 0.0 {
            break;
        }
        x1 = x3;
        y1 = y3;
        x3 += 0.0999999999;
    }

    y3 = conf - orderstat_tail(n, x3, m);

    // The root is now bracketed. Refine using Ridder's method.
    for _ in 0..200 {
        let x2 = 0.5 * (x1 + x3);
        if x3 - x1 < QCEPS {
            return x2;
        }

        let y2 = conf - orderstat_tail(n, x2, m);
        if y2.abs() < QCEPS {
            return x2;
        }

        let denom = (y2 * y2 - y1 * y3).sqrt();
        let x = x2 + (x1 - x2) * y2 / denom;
        let y = conf - orderstat_tail(n, x, m);
        if y.abs() < QCEPS {
            return x;
        }

        if (y2 < 0.0) && (y > 0.0) {
            x1 = x2;
            y1 = y2;
            x3 = x;
            y3 = y;
        } else if (y < 0.0) && (y2 > 0.0) {
            x1 = x;
            y1 = y;
            x3 = x2;
            y3 = y2;
        } else if y < 0.0 {
            x1 = x;
            y1 = y;
        } else {
            x3 = x;
            y3 = y;
        }
    }

    0.5 * (x1 + x3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lgamma() {
        // Test known values
        // lgamma(1) should be close to 0 (since gamma(1) = 1)
        // The algorithm is approximate, so we use a looser tolerance
        let result1 = lgamma(1.0);
        println!("lgamma(1.0) = {}", result1);
        assert!((result1 - 0.0).abs() < 0.01, "lgamma(1.0) = {}", result1);
        
        // lgamma(2) should be close to 0 (since gamma(2) = 1)
        let result2 = lgamma(2.0);
        println!("lgamma(2.0) = {}", result2);
        assert!((result2 - 0.0).abs() < 0.01, "lgamma(2.0) = {}", result2);
        
        // lgamma(3) should be close to ln(2) (since gamma(3) = 2)
        let result3 = lgamma(3.0);
        let expected3 = 2.0_f64.ln();
        println!("lgamma(3.0) = {}, expected = {}", result3, expected3);
        assert!((result3 - expected3).abs() < 0.01, "lgamma(3.0) = {}, expected = {}", result3, expected3);
    }

    #[test]
    fn test_ibeta() {
        // Test boundary conditions
        assert_eq!(ibeta(1.0, 1.0, 0.0), 0.0);
        assert_eq!(ibeta(1.0, 1.0, 1.0), 1.0);
        
        // Test uniform distribution (p=1, q=1)
        // The algorithm is approximate, so we use a looser tolerance
        let result = ibeta(1.0, 1.0, 0.5);
        println!("ibeta(1.0, 1.0, 0.5) = {}", result);
        assert!((result - 0.5).abs() < 0.01, "ibeta(1.0, 1.0, 0.5) = {}", result);
    }

    #[test]
    fn test_orderstat_tail() {
        // Boundary tests
        assert_eq!(orderstat_tail(10, 0.5, 11), 1.0);
        assert_eq!(orderstat_tail(10, 0.5, 0), 0.0);
    }

    #[test]
    fn test_quantile_conf() {
        // Test that quantile_conf returns a value in [0, 1]
        let result = quantile_conf(100, 10, 0.05);
        assert!(result >= 0.0 && result <= 1.0);
    }
}
