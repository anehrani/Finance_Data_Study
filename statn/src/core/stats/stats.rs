use std::f64::consts::PI;

const QCEPS: f64 = 1e-10;
const FPMIN: f64 = 1e-30;

// ============================================================================
// Normal CDF - Accurate to 7.5e-8
// ============================================================================

pub fn normal_cdf(z: f64) -> f64 {
    let zz = z.abs();
    let pdf = (-0.5 * zz * zz).exp() / (2.0 * PI).sqrt();
    let t = 1.0 / (1.0 + zz * 0.2316419);
    let poly = ((((1.330274429 * t - 1.821255978) * t + 1.781477937) * t
        - 0.356563782) * t
        + 0.319381530)
        * t;
    if z > 0.0 {
        1.0 - pdf * poly
    } else {
        pdf * poly
    }
}

// ============================================================================
// Inverse Normal CDF - Accurate to 4.5e-4
// ============================================================================

pub fn inverse_normal_cdf(p: f64) -> f64 {
    let pp = if p <= 0.5 { p } else { 1.0 - p };
    let t = (1.0 / (pp * pp)).ln().sqrt();
    let numer = (0.010328 * t + 0.802853) * t + 2.515517;
    let denom = ((0.001308 * t + 0.189269) * t + 1.432788) * t + 1.0;
    let x = t - numer / denom;

    if p <= 0.5 { -x } else { x }
}

// ============================================================================
// Complementary Error Function
// ============================================================================

pub fn erfc(x: f64) -> f64 {
    2.0 - 2.0 * normal_cdf(2.0_f64.sqrt() * x)
}

// ============================================================================
// Half-normal CDF
// ============================================================================

pub fn half_normal_cdf(s: f64) -> f64 {
    2.0 * normal_cdf(s) - 1.0
}

// ============================================================================
// Gamma (special case where argument is half-integer)
// ============================================================================

pub fn gamma_special(two_k: i32) -> f64 {
    let z = 0.5 * (two_k as f64);

    if two_k == 1 {
        PI.sqrt()
    } else if two_k == 2 {
        1.0
    } else {
        (z - 1.0) * gamma_special((2.0 * z - 1.9999) as i32)
    }
}

// ============================================================================
// Log Gamma (ACM algorithm 291)
// ============================================================================

pub fn lgamma(mut x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    let mut result = if x < 7.0 {
        let mut res = 1.0;
        let mut z = x;
        while z < 7.0 {
            res *= z;
            z += 1.0;
        }
        x = z;
        -res.ln()
    } else {
        0.0
    };


    let z = 1.0 / (x * x);

    result += (x - 0.5) * x.ln() - x + 0.918938533204673
        + ((((-0.000595238095238 * z + 0.000793650793651) * z - 0.002777777777778) * z
            + 0.083333333333333)
            / x);

    result
}

// ============================================================================
// Incomplete Gamma (From Press et al)
// ============================================================================

pub fn igamma(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    if x < a + 1.0 {
        let mut ap = a;
        let mut del = 1.0 / a;
        let mut sum = del;

        loop {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del < 1e-8 * sum {
                break;
            }
        }
        return sum * (a * x.ln() - x - lgamma(a)).exp();
    }

    let mut b = x + 1.0 - a;
    let mut c = 1.0 / FPMIN;
    let mut d = 1.0 / b;
    let mut h = d;

    for _i in 1..1000 {
        let an = (_i as f64) * (a - _i as f64);
        b += 2.0;
        d = an * d + b;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = b + an / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-8 {
            break;
        }
    }

    1.0 - h * (a * x.ln() - x - lgamma(a)).exp()
}

// ============================================================================
// Incomplete Beta
// ============================================================================

pub fn ibeta(mut p: f64, mut q: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if p <= 0.0 || q <= 0.0 {
        return -1.0;
    }

    let eps = 1e-12;
    let eps1: f64 = 1e-98;
    let aleps1 = eps1.ln();

    let switched_args = if x > 0.5 {
        let temp = p;
        p = q;
        q = temp;
        true
    } else {
        false
    };

    let x = if switched_args { 1.0 - x } else { x };

    let mut ps = q - q.floor();
    if ps == 0.0 {
        ps = 1.0;
    }

    let px = p * x.ln();
    let pq = lgamma(p + q);
    let p1 = lgamma(p);
    let d4 = p.ln();

    let mut term = px + lgamma(ps + p) - lgamma(ps) - d4 - p1;

    let infsum = if (term / aleps1).floor() == 0.0 {
        let mut infsum = (term).exp();
        let mut cnt = infsum * p;
        let mut wh = 1.0;

        loop {
            cnt *= (wh - ps) * x / wh;
            term = cnt / (p + wh);
            infsum += term;
            if term / eps <= infsum {
                break;
            }
            wh += 1.0;
        }
        infsum
    } else {
        0.0
    };

    let mut finsum = 0.0;
    if q > 1.0 {
        let xb = px + q * (1.0 - x).ln() + pq - p1 - q.ln() - lgamma(q);

        let mut ib = (xb / aleps1).floor() as i32;
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

// ============================================================================
// Inverse Incomplete Beta
// ============================================================================

pub fn inverse_ibeta(p: f64, a: f64, b: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }

    let am1 = a - 1.0;
    let bm1 = b - 1.0;
    let eps = 1e-8;

    let mut x = if a >= 1.0 && b >= 1.0 {
        let pp = if p < 0.5 { p } else { 1.0 - p };
        let t = (-2.0 * pp.ln()).sqrt();
        let mut x = (2.30753 + t * 0.27061) / (1.0 + t * (0.99229 + t * 0.04481)) - t;
        if p < 0.5 {
            x = -x;
        }
        let al = (x * x - 3.0) / 6.0;
        let h = 2.0 / (1.0 / (2.0 * a - 1.0) + 1.0 / (2.0 * b - 1.0));
        let w = (x * (al + h).sqrt() / h)
            - (1.0 / (2.0 * b - 1.0) - 1.0 / (2.0 * a - 1.0)) * (al + 5.0 / 6.0 - 2.0 / (3.0 * h));
        a / (a + b * (2.0 * w).exp())
    } else {
        let lna = (a / (a + b)).ln();
        let lnb = (b / (a + b)).ln();
        let t = (a * lna).exp() / a;
        let u = (b * lnb).exp() / b;
        let w = t + u;
        if p < t / w {
            (a * w * p).powf(1.0 / a)
        } else {
            1.0 - (b * w * (1.0 - p)).powf(1.0 / b)
        }
    };

    let afac = -lgamma(a) - lgamma(b) + lgamma(a + b);

    for _j in 0..10 {
        if x == 0.0 || x == 1.0 {
            return x;
        }

        let err = ibeta(a, b, x) - p;
        let t = (am1 * x.ln() + bm1 * (1.0 - x).ln() + afac).exp();
        let u = err / t;
        let delta = u / (1.0 - 0.5 * (u * (am1 / x - bm1 / (1.0 - x))).min(1.0));
        x -= delta;

        if x <= 0.0 {
            x = 0.5 * (x + delta);
        }
        if x >= 1.0 {
            x = 0.5 * (x + delta + 1.0);
        }

        if delta.abs() < eps * x {
            break;
        }
    }

    x
}

// ============================================================================
// Student's t CDF
// ============================================================================

pub fn t_cdf(ndf: i32, t: f64) -> f64 {
    let mut prob = 1.0 - 0.5 * ibeta(0.5 * (ndf as f64), 0.5, (ndf as f64) / ((ndf as f64) + t * t));
    prob = prob.max(0.0).min(1.0);
    if t >= 0.0 {
        prob
    } else {
        1.0 - prob
    }
}

// ============================================================================
// Inverse Student's t CDF
// ============================================================================

pub fn inverse_t_cdf(ndf: i32, p: f64) -> f64 {
    let x = inverse_ibeta(2.0 * p.min(1.0 - p), 0.5 * (ndf as f64), 0.5);
    let x = ((ndf as f64) * (1.0 - x) / x).sqrt();
    if p > 0.5 { x } else { -x }
}

// ============================================================================
// F CDF
// ============================================================================

pub fn f_cdf(ndf1: i32, ndf2: i32, f: f64) -> f64 {
    let mut prob = 1.0 - ibeta(0.5 * (ndf2 as f64), 0.5 * (ndf1 as f64), (ndf2 as f64) / ((ndf2 as f64) + (ndf1 as f64) * f));
    prob = prob.max(0.0).min(1.0);
    prob
}

// ============================================================================
// Poisson PDF
// ============================================================================

pub fn poisson_pdf(lambda: f64, k: i32) -> f64 {
    if k == 0 {
        (-lambda).exp()
    } else {
        (-lambda).exp() * lambda.powi(k) / gamma_special(2 * k + 2)
    }
}

// ============================================================================
// Anderson-Darling CDF
// ============================================================================

pub fn anderson_darling_cdf(z: f64) -> f64 {
    if z < 0.01 {
        0.0
    } else if z <= 2.0 {
        2.0 * (-1.2337 / z).exp() * (1.0 + z / 8.0 - 0.04958 * z * z / (1.325 + z)) / z.sqrt()
    } else if z <= 4.0 {
        1.0 - 0.6621361 * (-1.091638 * z).exp() - 0.95095 * (-2.005138 * z).exp()
    } else {
        1.0 - 0.4938691 * (-1.050321 * z).exp() - 0.5946335 * (-1.527198 * z).exp()
    }
}

// ============================================================================
// Kolmogorov-Smirnov CDF
// ============================================================================

pub fn ks_cdf(n: i32, dn: f64) -> f64 {
    if dn <= 0.0 || n <= 0 {
        return 0.0;
    }

    let mut arg = (n as f64).sqrt();
    arg = arg + 0.12 + 0.11 / arg;
    arg *= dn;
    arg = arg * arg;

    let mut sum = 0.0;

    for i in 1..100 {
        let term = -2.0 * (i as f64) * (i as f64) * arg;
        if term < -45.0 {
            break;
        }
        let term = term.exp();
        if i % 2 == 1 {
            sum += term;
        } else {
            sum -= term;
        }
    }

    sum = 1.0 - 2.0 * sum;
    sum = sum.max(0.0).min(1.0);
    sum
}

pub fn inverse_ks(n: i32, cdf: f64) -> f64 {
    (-0.5 * (1.0 - cdf).ln() / (2.0 * (n as f64))).sqrt()
}

// ============================================================================
// Student's t test for one sample
// ============================================================================

pub fn t_test_one_sample(x: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;

    let ss: f64 = x.iter().map(|xi| (xi - mean).powi(2)).sum();
    let std = (ss / (n * (n - 1.0))).sqrt();

    mean / (std + 1e-60)
}

// ============================================================================
// Student's t test for two samples
// ============================================================================

fn t_test_two_samples(x1: &[f64], x2: &[f64]) -> f64 {
    let n1 = x1.len() as f64;
    let n2 = x2.len() as f64;

    let mean1 = x1.iter().sum::<f64>() / n1;
    let mean2 = x2.iter().sum::<f64>() / n2;

    let ss1: f64 = x1.iter().map(|xi| (xi - mean1).powi(2)).sum();
    let ss2: f64 = x2.iter().map(|xi| (xi - mean2).powi(2)).sum();

    let std = ((ss1 + ss2) / (n1 + n2 - 2.0) * (1.0 / n1 + 1.0 / n2)).sqrt();

    (mean1 - mean2) / (std + 1e-60)
}

// ============================================================================
// Mann-Whitney U-test
// ============================================================================

pub fn u_test(x1: &[f64], x2: &[f64]) -> (f64, f64) {
    let n1 = x1.len();
    let n2 = x2.len();
    let n = n1 + n2;

    let mut combined: Vec<(f64, usize)> = x1
        .iter()
        .map(|&v| (v, 0))
        .chain(x2.iter().map(|&v| (v, 1)))
        .collect();

    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut ranks = vec![0.0; n];
    let mut tie_correc = 0.0;

    let mut j = 0;
    while j < n {
        let val = combined[j].0;
        let mut k = j + 1;
        while k < n && combined[k].0 == val {
            k += 1;
        }
        let ntied = k - j;
        tie_correc += (ntied as f64).powi(3) - ntied as f64;
        let rank = 0.5 * ((j as f64) + (k as f64) + 1.0);
        for idx in j..k {
            ranks[idx] = rank;
        }
        j = k;
    }

    let mut u = 0.0;
    for i in 0..n {
        if combined[i].1 == 0 {
            u += ranks[i];
        }
    }

    u = (n1 as f64) * (n2 as f64) + 0.5 * (n1 as f64) * ((n1 as f64) + 1.0) - u;

    let dn = n as f64;
    let term1 = (n1 as f64) * (n2 as f64) / (dn * (dn - 1.0));
    let term2 = (dn.powi(3) - dn - tie_correc) / 12.0;
    let z = (0.5 * (n1 as f64) * (n2 as f64) - u) / (term1 * term2).sqrt();

    (u, z)
}

// ============================================================================
// Kolmogorov-Smirnov test
// ============================================================================

pub fn ks_test(x: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let mut d_plus: f64 = 0.0;
    let mut d_minus: f64 = 0.0;
    let mut old_fn: f64 = 0.0;

    for (i, &xi) in x.iter().enumerate() {
        let fn_val = ((i as f64) + 1.0) / n;
        d_plus = d_plus.max(fn_val - xi);
        d_minus = d_minus.max(xi - old_fn);
        old_fn = fn_val;
    }

    (d_plus.max(d_minus), d_plus.max(d_minus))
}

// ============================================================================
// Anderson-Darling test
// ============================================================================

pub fn anderson_darling_test(mut x: Vec<f64>) -> f64 {
    let n = x.len();
    x.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut z = -1.0 * (n as f64).powi(2);
    for i in 0..n {
        let term = x[i] * (1.0 - x[n - i - 1]);
        let term = if term < 1e-30 { 1e-30 } else { term };
        z -= ((2.0 * (i as f64)) + 1.0) * term.ln();
    }

    1.0 - anderson_darling_cdf(z / (n as f64))
}

// ============================================================================
// One-way ANOVA
// ============================================================================

pub fn anova_1(x: &[f64], group_ids: &[usize], num_groups: usize) -> (f64, f64, f64) {
    let n = x.len();
    let mut counts = vec![0; num_groups];
    let mut sums = vec![0.0; num_groups];
    let mut grand_sum = 0.0;

    for (i, &gid) in group_ids.iter().enumerate() {
        counts[gid] += 1;
        sums[gid] += x[i];
        grand_sum += x[i];
    }

    let grand_mean = grand_sum / (n as f64);
    let mut means = vec![0.0; num_groups];
    for k in 0..num_groups {
        means[k] = sums[k] / (counts[k] as f64 + 1e-60);
    }

    let mut between = 0.0;
    for k in 0..num_groups {
        let diff = means[k] - grand_mean;
        between += (counts[k] as f64) * diff * diff;
    }
    between /= (num_groups as f64 - 1.0).max(1.0);

    let mut within = 0.0;
    for (i, &gid) in group_ids.iter().enumerate() {
        let diff = x[i] - means[gid];
        within += diff * diff;
    }
    within /= (n as f64 - num_groups as f64).max(1.0);

    let f_ratio = between / (within + 1e-60);
    let pval = 1.0 - f_cdf((num_groups - 1) as i32, (n - num_groups) as i32, f_ratio);
    let account = between / (between + within + 1e-60);

    (f_ratio, account, pval)
}

// ============================================================================
// Kruskal-Wallis test
// ============================================================================

pub fn kruskal_wallis(x: &[f64], group_ids: &[usize], num_groups: usize) -> f64 {
    let n = x.len();
    let mut combined: Vec<(f64, usize)> = x
        .iter()
        .zip(group_ids.iter())
        .map(|(&v, &gid)| (v, gid))
        .collect();

    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut ranks = vec![0.0; n];
    let mut tie_correc = 0.0;

    let mut j = 0;
    while j < n {
        let val = combined[j].0;
        let mut k = j + 1;
        while k < n && combined[k].0 == val {
            k += 1;
        }
        let ntied = k - j;
        tie_correc += (ntied as f64).powi(3) - ntied as f64;
        let rank = 0.5 * ((j as f64) + (k as f64) + 1.0);
        for idx in j..k {
            ranks[idx] = rank;
        }
        j = k;
    }

    let mut kw = 0.0;
    for k in 0..num_groups {
        let mut nn = 0;
        let mut sum = 0.0;
        for (i, gid) in combined.iter().map(|(_, g)| *g).enumerate() {
            if gid == k {
                nn += 1;
                sum += ranks[i];
            }
        }
        kw += (sum + nn as f64).powi(2) / (nn as f64 + 1e-60);
    }

    kw = 12.0 / ((n as f64) * (n as f64 + 1.0)) * kw - 3.0 * (n as f64 + 1.0);
    kw /= 1.0 - tie_correc / ((n as f64).powi(3) - n as f64);

    kw
}

// ============================================================================
// Chi-square test
// ============================================================================

pub fn chisq(data: &[Vec<i32>]) -> (f64, f64, f64, f64) {
    let nrows = data.len();
    let ncols = if nrows > 0 { data[0].len() } else { 0 };

    let ndf = ((nrows - 1) * (ncols - 1)) as i32;
    if ndf == 0 {
        return (0.0, 0.0, 0.0, 1.0);
    }

    let mut row_margins = vec![0; nrows];
    let mut col_margins = vec![0; ncols];
    let mut total = 0;

    for (i, row) in data.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            row_margins[i] += val;
            col_margins[j] += val;
            total += val;
        }
    }

    let mut chi_square = 0.0;
    for (i, row) in data.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            let expected = (row_margins[i] as f64) * (col_margins[j] as f64) / (total as f64 + 1e-20);
            let diff = (val as f64) - expected;
            chi_square += diff * diff / (expected + 1e-20);
        }
    }

    let contin = (chi_square / ((total as f64) + chi_square)).sqrt();
    let mut cramer_v = chi_square / (total as f64);
    cramer_v /= if nrows < ncols {
        (nrows - 1) as f64
    } else {
        (ncols - 1) as f64
    };
    cramer_v = cramer_v.sqrt();

    let pval = 1.0 - igamma(0.5 * (ndf as f64), 0.5 * chi_square);

    (chi_square, contin, cramer_v, pval)
}

// ============================================================================
// Nominal Lambda
// ============================================================================

pub fn nominal_lambda(data: &[Vec<i32>]) -> (f64, f64, f64) {
    let nrows = data.len();
    let ncols = if nrows > 0 { data[0].len() } else { 0 };

    if nrows < 2 || ncols < 2 {
        return (0.0, 0.0, 0.0);
    }

    let mut sum_row_cell_max = 0;
    let mut max_row_total = 0;
    let mut total = 0;

    for row in data.iter() {
        let mut row_cell_max = 0;
        let mut row_total = 0;
        for &val in row.iter() {
            row_cell_max = row_cell_max.max(val);
            row_total += val;
            total += val;
        }
        max_row_total = max_row_total.max(row_total);
        sum_row_cell_max += row_cell_max;
    }

    let mut sum_col_cell_max = 0;
    let mut max_col_total = 0;

    for j in 0..ncols {
        let mut col_cell_max = 0;
        let mut col_total = 0;
        for i in 0..nrows {
            let val = data[i][j];
            col_cell_max = col_cell_max.max(val);
            col_total += val;
        }
        max_col_total = max_col_total.max(col_total);
        sum_col_cell_max += col_cell_max;
    }

    let row_dep = if total > max_row_total {
        ((sum_col_cell_max - max_row_total) as f64) / ((total - max_row_total) as f64)
    } else {
        1.0
    };

    let col_dep = if total > max_col_total {
        ((sum_row_cell_max - max_col_total) as f64) / ((total - max_col_total) as f64)
    } else {
        1.0
    };

    let numer = sum_col_cell_max - max_row_total + sum_row_cell_max - max_col_total;
    let denom = 2 * total - max_row_total - max_col_total;
    let sym = if denom > 0 {
        (numer as f64) / (denom as f64)
    } else {
        1.0
    };

    (row_dep, col_dep, sym)
}

// ============================================================================
// Uncertainty Reduction
// ============================================================================

pub fn uncertainty_reduction(data: &[Vec<i32>]) -> (f64, f64, f64) {
    let nrows = data.len();
    let ncols = if nrows > 0 { data[0].len() } else { 0 };

    if nrows < 2 || ncols < 2 {
        return (0.0, 0.0, 0.0);
    }

    let mut row_margins = vec![0; nrows];
    let mut col_margins = vec![0; ncols];
    let mut total = 0;

    for (i, row) in data.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            row_margins[i] += val;
            col_margins[j] += val;
            total += val;
        }
    }

    let mut u_row = 0.0;
    for &rm in row_margins.iter() {
        if rm > 0 {
            let p = (rm as f64) / (total as f64);
            u_row -= p * p.ln();
        }
    }

    let mut u_col = 0.0;
    for &cm in col_margins.iter() {
        if cm > 0 {
            let p = (cm as f64) / (total as f64);
            u_col -= p * p.ln();
        }
    }

    let mut u_sym = 0.0;
    for row in data.iter() {
        for &val in row.iter() {
            if val > 0 {
                let p = (val as f64) / (total as f64);
                u_sym -= p * p.ln();
            }
        }
    }

    let u_row_given_col = u_sym - u_col;
    let u_col_given_row = u_sym - u_row;

    let u_row_red = (u_row - u_row_given_col) / (u_row + 1e-60);
    let u_col_red = (u_col - u_col_given_row) / (u_col + 1e-60);
    let u_sym_red = 2.0 * (u_row + u_col - u_sym) / (u_row + u_col + 1e-60);

    (u_row_red, u_col_red, u_sym_red)
}

pub fn find_quantile(sorted_data: &[f64], fractile: f64) -> f64 {
    let n = sorted_data.len();
    let mut k = ((fractile * (n as f64 + 1.0)) as usize).saturating_sub(1);
    if k >= n {
        k = n - 1;
    }
    sorted_data[k]
}

pub fn find_min_max(data: &[f64]) -> (f64, f64) {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for &val in data {
        if val < min_val {
            min_val = val;
        }
        if val > max_val {
            max_val = val;
        }
    }

    (min_val, max_val)
}

// ============================================================================
// Left Binomial
// ============================================================================

pub fn left_binomial(n: i32, p: f64, m: i32) -> f64 {
    if m >= n {
        1.0
    } else if m < 0 {
        0.0
    } else {
        1.0 - ibeta((m + 1) as f64, (n - m) as f64, p)
    }
}

// ============================================================================
// Combinations
// ============================================================================

pub fn combinations(mut n: i32, mut m: i32) -> f64 {
    if m < n - m {
        // Keep m as is
    } else {
        m = n - m;
    }

    let mut product = 1.0;
    while m > 0 {
        product *= (n as f64) / (m as f64);
        n -= 1;
        m -= 1;
    }
    product
}

// ============================================================================
// Order statistic tail
// ============================================================================

pub fn orderstat_tail(n: i32, q: f64, m: i32) -> f64 {
    if m > n {
        1.0
    } else if m <= 0 {
        0.0
    } else {
        1.0 - ibeta(m as f64, (n - m + 1) as f64, q)
    }
}

// ============================================================================
// Quantile confidence
// ============================================================================

pub fn quantile_conf(n: i32, m: i32, conf: f64) -> f64 {
    let mut x1 = 0.0;
    let mut y1 = conf - 1.0;
    let mut x3 = 0.1;
    let mut y3;

    // Find bracketing interval
    loop {
        if x3 > 1.0 {
            x3 = 1.0;
        }
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

        if x3 > 1.0 {
            x3 = 1.0;
            break;
        }
    }

    // Ridder's method
    let mut x2 = 0.0;
    let mut y2 = 0.0;

    for _iter in 0..200 {
        x2 = 0.5 * (x1 + x3);
        if (x3 - x1).abs() < QCEPS {
            return x2;
        }

        y2 = conf - orderstat_tail(n, x2, m);
        if y2.abs() < QCEPS {
            return x2;
        }

        let denom = (y2 * y2 - y1 * y3).sqrt();
        if denom == 0.0 {
            break;
        }

        let x = x2 + (x1 - x2) * y2 / denom;
        let y = conf - orderstat_tail(n, x, m);
        if y.abs() < QCEPS {
            return x;
        }

        if (y2 < 0.0 && y > 0.0) || (y < 0.0 && y2 > 0.0) {
            x1 = x2.min(x);
            y1 = y2;
            x3 = x2.max(x);
            y3 = y;
        } else if y < 0.0 {
            x1 = x;
            y1 = y;
        } else {
            x3 = x;
            y3 = y;
        }
    }

    x2
}

// ============================================================================
// ROC Area
// ============================================================================

pub fn roc_area(pred: &mut [f64], target: &mut [f64], center: bool) -> f64 {
    let n = pred.len();

    if center {
        let mean = target.iter().sum::<f64>() / (n as f64);
        for t in target.iter_mut() {
            *t -= mean;
        }
    }

    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&i, &j| pred[i].partial_cmp(&pred[j]).unwrap());

    let mut reordered_pred = vec![0.0; n];
    let mut reordered_target = vec![0.0; n];
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        reordered_pred[new_idx] = pred[old_idx];
        reordered_target[new_idx] = target[old_idx];
    }

    let mut win_sum = 0.0;
    let mut lose_sum = 0.0;

    for &t in reordered_target.iter() {
        if t > 0.0 {
            win_sum += t;
        } else {
            lose_sum -= t;
        }
    }

    if win_sum == 0.0 || lose_sum == 0.0 {
        return 0.5;
    }

    let mut win = 0.0;
    let mut roc = 0.0;

    for i in (0..n).rev() {
        if reordered_target[i] > 0.0 {
            win += reordered_target[i] / win_sum;
        } else if reordered_target[i] < 0.0 {
            roc -= win * reordered_target[i] / lose_sum;
        }
    }

    roc
}

// ============================================================================
// Online Moments
// ============================================================================

pub struct OnlineStats {
    n: i64,
    delta: Vec<f64>,
    mean: Vec<f64>,
    sum2: Vec<f64>,
    sum3: Vec<f64>,
    sum4: Vec<f64>,
}

impl OnlineStats {
    pub fn new(num_streams: usize) -> Self {
        OnlineStats {
            n: 0,
            delta: vec![0.0; num_streams],
            mean: vec![0.0; num_streams],
            sum2: vec![0.0; num_streams],
            sum3: vec![0.0; num_streams],
            sum4: vec![0.0; num_streams],
        }
    }

    pub fn update(&mut self, y: &[f64]) {
        if self.n == 0 {
            for i in 0..y.len() {
                self.mean[i] = y[i];
                self.sum2[i] = 0.0;
                self.sum3[i] = 0.0;
                self.sum4[i] = 0.0;
            }
            self.n = 1;
            return;
        }

        let np1 = (self.n + 1) as f64;

        for i in 0..y.len() {
            self.delta[i] = y[i] - self.mean[i];
            self.mean[i] += self.delta[i] / np1;
            let dsquare = self.delta[i] * self.delta[i];

            let n_f = self.n as f64;
            self.sum4[i] += n_f * (n_f * n_f - n_f + 1.0) * dsquare * dsquare / (np1 * np1 * np1);
            self.sum4[i] += 6.0 * self.sum2[i] * dsquare / (np1 * np1);
            self.sum4[i] -= 4.0 * self.sum3[i] * self.delta[i] / np1;
            self.sum3[i] += n_f * (n_f - 1.0) * dsquare * self.delta[i] / (np1 * np1);
            self.sum3[i] -= 3.0 * self.sum2[i] * self.delta[i] / np1;
            self.sum2[i] += n_f * dsquare / np1;
        }

        self.n += 1;
    }

    pub fn get_mean(&self) -> Vec<f64> {
        self.mean.clone()
    }

    pub fn get_variance(&self) -> Vec<f64> {
        self.sum2.iter().map(|&s| s / (self.n as f64)).collect()
    }

    pub fn get_skewness(&self) -> Vec<f64> {
        let mut result = Vec::new();
        for &s2 in self.sum2.iter() {
            let std = (s2 / (self.n as f64)).sqrt();
            let idx = self.sum2.len() - result.len() - 1;
            result.push(self.sum3[idx] / ((self.n as f64) * std * std * std));
        }
        result
    }

    pub fn get_kurtosis(&self) -> Vec<f64> {
        let mut result = Vec::new();
        for (idx, &s2) in self.sum2.iter().enumerate() {
            let var = s2 / (self.n as f64);
            result.push(self.sum4[idx] / ((self.n as f64) * var * var));
        }
        result
    }
}



/*
Compute relative entropy
*/

pub fn entropy(data: &[f64], nbins: usize) -> f64 {
    let n = data.len();
    if n == 0 || nbins < 2 {
        return 0.0;
    }

    let minval = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let maxval = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let factor = (nbins as f64 - 1e-10) / (maxval - minval + 1e-60);

    let mut count = vec![0; nbins];

    for &x in data {
        let k = ((factor * (x - minval)) as usize).min(nbins - 1);
        count[k] += 1;
    }

    let mut sum = 0.0;
    for &c in &count {
        if c > 0 {
            let p = c as f64 / n as f64;
            sum += p * p.ln();
        }
    }

    -sum / (nbins as f64).ln()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        assert!(normal_cdf(2.0) > 0.97);
        assert!(normal_cdf(-2.0) < 0.03);
    }

    #[test]
    fn test_inverse_normal_cdf() {
        let p = 0.975;
        let z = inverse_normal_cdf(p);
        assert!((normal_cdf(z) - p).abs() < 1e-4);
    }

    #[test]
    fn test_t_test_one_sample() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let t = t_test_one_sample(&x);
        assert!(t.is_finite());
    }

    #[test]
    fn test_combinations() {
        assert!((combinations(5, 2) - 10.0).abs() < 1e-10);
        assert!((combinations(10, 3) - 120.0).abs() < 1e-10);
    }

    #[test]
    fn test_online_stats() {
        let mut stats = OnlineStats::new(1);
        for i in 1..=5 {
            stats.update(&[i as f64]);
        }
        let mean = stats.get_mean();
        assert!((mean[0] - 3.0).abs() < 1e-10);
    }
}