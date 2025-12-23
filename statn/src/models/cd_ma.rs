use std::fs::OpenOptions;
use std::io::Write;

const RESULTS: bool = false;

use serde::{Deserialize, Serialize};

/// Coordinate Descent model for elastic net regularized regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateDescent {
    // Public fields
    pub ok: bool,
    pub beta: Vec<f64>,
    pub explained: f64,
    pub xmeans: Vec<f64>,
    pub xscales: Vec<f64>,
    pub ymean: f64,
    pub yscale: f64,

    // Private fields
    nvars: usize,
    ncases: usize,
    covar_updates: bool,
    n_lambda: usize,
    #[serde(skip, default)]
    lambda_beta: Vec<f64>,
    #[serde(skip, default)]
    lambdas: Vec<f64>,
    #[serde(skip, default)]
    x: Vec<f64>,
    #[serde(skip, default)]
    y: Vec<f64>,
    #[serde(skip, default)]
    w: Option<Vec<f64>>,
    #[serde(skip, default)]
    resid: Vec<f64>,
    #[serde(skip, default)]
    xinner: Option<Vec<f64>>,
    #[serde(skip, default)]
    yinner: Option<Vec<f64>>,
    #[serde(skip, default)]
    xssvec: Option<Vec<f64>>,
}

impl CoordinateDescent {
    /// Constructor
    pub fn new(
        nvars: usize,
        ncases: usize,
        weighted: bool,
        covar_updates: bool,
        n_lambda: usize,
    ) -> Self {
        let mut cd = CoordinateDescent {
            ok: true,
            nvars,
            ncases,
            covar_updates,
            n_lambda,
            beta: vec![0.0; nvars],
            explained: 0.0,
            xmeans: vec![0.0; nvars],
            xscales: vec![0.0; nvars],
            ymean: 0.0,
            yscale: 0.0,
            lambda_beta: if n_lambda > 0 {
                vec![0.0; n_lambda * nvars]
            } else {
                Vec::new()
            },
            lambdas: if n_lambda > 0 {
                vec![0.0; n_lambda]
            } else {
                Vec::new()
            },
            x: vec![0.0; ncases * nvars],
            y: vec![0.0; ncases],
            w: if weighted {
                Some(vec![0.0; ncases])
            } else {
                None
            },
            resid: vec![0.0; ncases],
            xinner: if covar_updates {
                Some(vec![0.0; nvars * nvars])
            } else {
                None
            },
            yinner: if covar_updates {
                Some(vec![0.0; nvars])
            } else {
                None
            },
            xssvec: if weighted {
                Some(vec![0.0; nvars])
            } else {
                None
            },
        };

        // Validate allocations
        if cd.x.is_empty() || cd.y.is_empty() || cd.xmeans.is_empty() || cd.xscales.is_empty()
            || cd.beta.is_empty() || cd.resid.is_empty()
            || (weighted && cd.w.is_none())
            || (covar_updates && cd.xinner.is_none())
            || (covar_updates && cd.yinner.is_none())
            || (n_lambda > 0 && cd.lambda_beta.is_empty())
            || (n_lambda > 0 && cd.lambdas.is_empty())
        {
            cd.ok = false;
        }

        cd
    }

    /// Get and standardize the data
    pub fn get_data(
        &mut self,
        istart: usize,
        n: usize,
        xx: &[f64],
        yy: &[f64],
        ww: Option<&[f64]>,
    ) {
        // Standardize X
        for ivar in 0..self.nvars {
            let mut xm = 0.0;
            for icase in 0..self.ncases {
                let k = (icase + istart) % n;
                xm += xx[k * self.nvars + ivar];
            }
            xm /= self.ncases as f64;
            self.xmeans[ivar] = xm;

            let mut xs = 1.0e-60;
            for icase in 0..self.ncases {
                let k = (icase + istart) % n;
                let diff = xx[k * self.nvars + ivar] - xm;
                xs += diff * diff;
            }
            xs = (xs / self.ncases as f64).sqrt();
            self.xscales[ivar] = xs;

            for icase in 0..self.ncases {
                let k = (icase + istart) % n;
                self.x[icase * self.nvars + ivar] =
                    (xx[k * self.nvars + ivar] - xm) / xs;
            }
        }

        // Standardize Y
        self.ymean = 0.0;
        for icase in 0..self.ncases {
            let k = (icase + istart) % n;
            self.ymean += yy[k];
        }
        self.ymean /= self.ncases as f64;

        let mut yscale = 1.0e-60;
        for icase in 0..self.ncases {
            let k = (icase + istart) % n;
            let diff = yy[k] - self.ymean;
            yscale += diff * diff;
        }
        yscale = (yscale / self.ncases as f64).sqrt();
        self.yscale = yscale;

        for icase in 0..self.ncases {
            let k = (icase + istart) % n;
            self.y[icase] = (yy[k] - self.ymean) / yscale;
        }

        // Handle weights if present
        if let Some(ref mut w) = self.w 
            && let Some(ww_data) = ww {
            let mut sum = 0.0;
            for (icase, w_val) in w.iter_mut().enumerate().take(self.ncases) {
                let k = (icase + istart) % n;
                *w_val = ww_data[k];
                sum += *w_val;
            }
            for w_val in w.iter_mut().take(self.ncases) {
                *w_val /= sum;
            }

            // Compute weighted X sum of squares
            if let Some(ref mut xssvec) = self.xssvec {
                for (ivar, xss_val) in xssvec.iter_mut().enumerate().take(self.nvars) {
                    let mut sum = 0.0;
                    for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                        let x_val = self.x[icase * self.nvars + ivar];
                        sum += weight * x_val * x_val;
                    }
                    *xss_val = sum;
                }
            }
        }

        // Compute inner products if using covariance updates
        if self.covar_updates 
            && let (Some(xinner), Some(yinner)) = (&mut self.xinner, &mut self.yinner) {
                for ivar in 0..self.nvars {
                    // Compute XiY
                    let mut sum = 0.0;
                    if let Some(ref w) = self.w {
                        for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                            sum += weight * self.x[icase * self.nvars + ivar] * self.y[icase];
                        }
                        yinner[ivar] = sum;
                    } else {
                        for icase in 0..self.ncases {
                            sum += self.x[icase * self.nvars + ivar] * self.y[icase];
                        }
                        yinner[ivar] = sum / self.ncases as f64;
                    }

                    // Compute XiXj
                    if let Some(ref w) = self.w {
                        for jvar in 0..self.nvars {
                            if jvar == ivar {
                                xinner[ivar * self.nvars + jvar] = self.xssvec.as_ref().unwrap()[ivar];
                            } else if jvar < ivar {
                                xinner[ivar * self.nvars + jvar] =
                                    xinner[jvar * self.nvars + ivar];
                            } else {
                                let mut sum = 0.0;
                                for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                                    sum += weight
                                        * self.x[icase * self.nvars + ivar]
                                        * self.x[icase * self.nvars + jvar];
                                }
                                xinner[ivar * self.nvars + jvar] = sum;
                            }
                        }
                    } else {
                        for jvar in 0..self.nvars {
                            if jvar == ivar {
                                xinner[ivar * self.nvars + jvar] = 1.0;
                            } else if jvar < ivar {
                                xinner[ivar * self.nvars + jvar] =
                                    xinner[jvar * self.nvars + ivar];
                            } else {
                                let mut sum = 0.0;
                                for icase in 0..self.ncases {
                                    sum += self.x[icase * self.nvars + ivar]
                                        * self.x[icase * self.nvars + jvar];
                                }
                                xinner[ivar * self.nvars + jvar] = sum / self.ncases as f64;
                            }
                        }
                    }
                }
            }

    }

    /// Core training routine using coordinate descent
    pub fn core_train(
        &mut self,
        alpha: f64,
        lambda: f64,
        maxits: usize,
        eps: f64,
        fast_test: bool,
        warm_start: bool,
    ) {
        let s_threshold = alpha * lambda;
        let mut do_active_only = false;
        let mut prior_crit = 1.0e60;

        // Initialize betas and residuals
        if warm_start {
            if !self.covar_updates {
                for icase in 0..self.ncases {
                    let mut sum = 0.0;
                    for ivar in 0..self.nvars {
                        sum += self.beta[ivar] * self.x[icase * self.nvars + ivar];
                    }
                    self.resid[icase] = self.y[icase] - sum;
                }
            }
        } else {
            self.beta.iter_mut().for_each(|b| *b = 0.0);
            self.resid.copy_from_slice(&self.y);
        }

        // Compute YmeanSquare for variance calculation
        let ymean_square = if self.w.is_some() {
            let w = self.w.as_ref().unwrap();
            self.y.iter().enumerate()
                .map(|(i, &y_val)| w[i] * y_val * y_val)
                .sum::<f64>()
        } else {
            1.0
        };

        // Main iteration loop
        for _iter in 0..maxits {
            let mut active_set_changed = false;
            let mut max_change = 0.0;

            // Process each variable
            for ivar in 0..self.nvars {
                if do_active_only && self.beta[ivar] == 0.0 {
                    continue;
                }

                // Compute update factor (denominator)
                let xss = if self.w.is_some() {
                    self.xssvec.as_ref().unwrap()[ivar]
                } else {
                    1.0
                };
                let update_factor = xss + lambda * (1.0 - alpha);

                // Compute argument to soft-thresholding operator
                let argument = if self.covar_updates {
                    let xinner = self.xinner.as_ref().unwrap();
                    let yinner = self.yinner.as_ref().unwrap();
                    let mut sum = 0.0;
                    for kvar in 0..self.nvars {
                        sum += xinner[ivar * self.nvars + kvar] * self.beta[kvar];
                    }
                    let residual_sum = yinner[ivar] - sum;
                    residual_sum + xss * self.beta[ivar]
                } else if self.w.is_some() {
                    let w = self.w.as_ref().unwrap();
                    let mut sum = 0.0;
                    for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                        let x_val = self.x[icase * self.nvars + ivar];
                        sum += weight
                            * x_val
                            * (self.resid[icase] + self.beta[ivar] * x_val);
                    }
                    sum
                } else {
                    let mut residual_sum = 0.0;
                    for icase in 0..self.ncases {
                        residual_sum += self.x[icase * self.nvars + ivar] * self.resid[icase];
                    }
                    residual_sum / self.ncases as f64 + self.beta[ivar]
                };

                // Apply soft-thresholding operator S()
                let new_beta = if argument > 0.0 && s_threshold < argument {
                    (argument - s_threshold) / update_factor
                } else if argument < 0.0 && s_threshold < -argument {
                    (argument + s_threshold) / update_factor
                } else {
                    0.0
                };

                // Update beta and residuals
                let correction = new_beta - self.beta[ivar];
                if correction.abs() > max_change {
                    max_change = correction.abs();
                }

                if correction != 0.0 {
                    if !self.covar_updates {
                        for icase in 0..self.ncases {
                            self.resid[icase] -=
                                correction * self.x[icase * self.nvars + ivar];
                        }
                    }
                    if (self.beta[ivar] == 0.0 && new_beta != 0.0)
                        || (self.beta[ivar] != 0.0 && new_beta == 0.0)
                    {
                        active_set_changed = true;
                    }
                    self.beta[ivar] = new_beta;
                }
            }

            // Check convergence
            let converged = if fast_test {
                max_change < eps
            } else {
                // Compute explained variance
                if self.covar_updates {
                    for icase in 0..self.ncases {
                        let mut sum = 0.0;
                        for ivar in 0..self.nvars {
                            sum += self.beta[ivar] * self.x[icase * self.nvars + ivar];
                        }
                        self.resid[icase] = self.y[icase] - sum;
                    }
                }

                let mut sum = 0.0;
                let crit = if let Some(ref w) = self.w {
                    for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                        sum += weight * self.resid[icase] * self.resid[icase];
                    }
                    sum
                } else {
                    for i in 0..self.ncases {
                        sum += self.resid[i] * self.resid[i];
                    }
                    sum / self.ncases as f64
                };

                let mut penalty = 0.0;
                for i in 0..self.nvars {
                    penalty += 0.5 * (1.0 - alpha) * self.beta[i] * self.beta[i]
                        + alpha * self.beta[i].abs();
                }
                penalty *= 2.0 * lambda;

                let crit = crit + penalty;

                if prior_crit - crit < eps {
                    true
                } else {
                    prior_crit = crit;
                    false
                }
            };

            // Update active set strategy
            if do_active_only {
                if converged {
                    do_active_only = false;
                }
            } else {
                if converged && !active_set_changed {
                    break;
                }
                do_active_only = true;
            }
        }

        // Compute final explained variance
        if fast_test && self.covar_updates {
            for icase in 0..self.ncases {
                let mut sum = 0.0;
                for ivar in 0..self.nvars {
                    sum += self.beta[ivar] * self.x[icase * self.nvars + ivar];
                }
                self.resid[icase] = self.y[icase] - sum;
            }
        }

        let mut sum = 0.0;
        let crit = if let Some(ref w) = self.w {
            for (i, &weight) in w.iter().enumerate().take(self.ncases) {
                sum += weight * self.resid[i] * self.resid[i];
            }
            sum
        } else {
            for i in 0..self.ncases {
                sum += self.resid[i] * self.resid[i];
            }
            sum / self.ncases as f64
        };

        self.explained = (ymean_square - crit) / ymean_square;
    }

    /// Get minimum lambda such that all betas remain at zero
    pub fn get_lambda_thresh(&self, alpha: f64) -> f64 {
        let mut thresh = 0.0;
        for ivar in 0..self.nvars {
            let mut sum = 0.0;
            if let Some(ref w) = self.w {
                for (icase, &weight) in w.iter().enumerate().take(self.ncases) {
                    sum += weight * self.x[icase * self.nvars + ivar] * self.y[icase];
                }
            } else {
                for icase in 0..self.ncases {
                    sum += self.x[icase * self.nvars + ivar] * self.y[icase];
                }
                sum /= self.ncases as f64;
            }
            sum = sum.abs();
            if sum > thresh {
                thresh = sum;
            }
        }
        thresh / (alpha + 1.0e-60)
    }

    /// Training with multiple lambdas
    pub fn lambda_train(
        &mut self,
        alpha: f64,
        maxits: usize,
        eps: f64,
        fast_test: bool,
        mut max_lambda: f64,
        print_steps: bool,
    ) {
        if self.n_lambda <= 1 {
            return;
        }

        if max_lambda <= 0.0 {
            max_lambda = 0.999 * self.get_lambda_thresh(alpha);
        }

        let min_lambda = 0.001 * max_lambda;
        let lambda_factor = ((min_lambda / max_lambda).ln() / (self.n_lambda - 1) as f64).exp();

        if print_steps 
            && let Ok(mut file) = OpenOptions::new().create(true).append(true).open("CDtest.LOG") {
                let _ = writeln!(file, "\n\nDescending lambda training...");
                let _ = writeln!(file, "Lambda  n_active  Explained");
        }

        let mut lambda = max_lambda;
        for ilambda in 0..self.n_lambda {
            self.lambdas[ilambda] = lambda;
            self.core_train(alpha, lambda, maxits, eps, fast_test, ilambda > 0);

            for ivar in 0..self.nvars {
                self.lambda_beta[ilambda * self.nvars + ivar] = self.beta[ivar];
            }

            if print_steps {
                let n_active = self.beta.iter().filter(|&&b| b != 0.0).count();
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("CDtest.LOG") {
                    let _ = writeln!(
                        file,
                        "\n{:8.4} {:4} {:12.4}",
                        lambda, n_active, self.explained
                    );
                }
            }

            lambda *= lambda_factor;
        }
    }
}

/// Cross-validation training routine
#[allow(clippy::too_many_arguments)]
pub fn cv_train(
    nvars: usize,
    nfolds: usize,
    xx: &[f64],
    yy: &[f64],
    ww: Option<&[f64]>,
    lambdas: &mut [f64],
    lambda_oos: &mut [f64],
    covar_updates: bool,
    n_lambda: usize,
    alpha: f64,
    maxits: usize,
    eps: f64,
    fast_test: bool,
) -> f64 {
    let n = yy.len();

    if n_lambda < 2 {
        return 0.0;
    }

    let mut work = vec![0.0; n];

    // Use entire dataset to find max lambda
    let mut cd = CoordinateDescent::new(nvars, n, ww.is_some(), covar_updates, n_lambda);
    cd.get_data(0, n, xx, yy, ww);
    let max_lambda = cd.get_lambda_thresh(alpha);

    if let Some(_ww_data) = ww 
        && let Some(ref w) = cd.w {
            work[..n].copy_from_slice(&w[..n]);
    }

    if RESULTS 
        && let Ok(mut file) = OpenOptions::new().create(true).append(true).open("CDtest.LOG") {
            let _ = writeln!(
                file,
                "\n\n\ncv_train() starting for {} folds with max lambda={:.4}\n",
                nfolds, max_lambda
            );
    }

    let mut i_is = 0;
    let mut n_done = 0;

    for val in lambda_oos.iter_mut().take(n_lambda) {
        *val = 0.0;
    }

    let mut yssum_squares = 0.0;

    // Process folds
    for _ifold in 0..nfolds {
        let n_oos = (n - n_done) / (nfolds - _ifold);
        let n_is = n - n_oos;
        let i_oos = (i_is + n_is) % n;

        // Train model with IS set
        let mut cd_fold = CoordinateDescent::new(nvars, n_is, ww.is_some(), covar_updates, n_lambda);
        cd_fold.get_data(i_is, n, xx, yy, ww);
        cd_fold.lambda_train(alpha, maxits, eps, fast_test, max_lambda, false);

        // Compute OOS performance for each lambda
        for ilambda in 0..n_lambda {
            lambdas[ilambda] = cd_fold.lambdas[ilambda];
            let coefs = &cd_fold.lambda_beta[ilambda * nvars..(ilambda + 1) * nvars];

            let mut sum = 0.0;
            for icase in 0..n_oos {
                let k = (icase + i_oos) % n;
                let mut pred = 0.0;
                for ivar in 0..nvars {
                    pred += coefs[ivar] * (xx[k * nvars + ivar] - cd_fold.xmeans[ivar])
                        / cd_fold.xscales[ivar];
                }

                let ynormalized = (yy[k] - cd_fold.ymean) / cd_fold.yscale;
                let diff = ynormalized - pred;

                if let Some(ww_data) = ww {
                    if ilambda == 0 {
                        yssum_squares += ww_data[k] * ynormalized * ynormalized;
                    }
                    sum += ww_data[k] * diff * diff;
                } else {
                    if ilambda == 0 {
                        yssum_squares += ynormalized * ynormalized;
                    }
                    sum += diff * diff;
                }
            }
            lambda_oos[ilambda] += sum;
        }

        n_done += n_oos;
        i_is = (i_is + n_oos) % n;
    }

    // Compute OOS explained variance for each lambda
    let mut best = -1.0e60;
    let mut ibest = 0;

    for (ilambda, val) in lambda_oos.iter_mut().enumerate().take(n_lambda) {
        *val = (yssum_squares - *val) / yssum_squares;
        if *val > best {
            best = *val;
            ibest = ilambda;
        }
    }

    if RESULTS 
        && let Ok(mut file) = OpenOptions::new().create(true).append(true).open("CDtest.LOG") {
            let _ = writeln!(
                file,
                "\ncv_train() ending with best lambda={:.4}  explained={:.4}",
                lambdas[ibest], best
            );
    }

    lambdas[ibest]
}