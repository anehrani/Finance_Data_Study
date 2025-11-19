use std::f64;

const EPSILON: f64 = 1e-60;

/// Singular Value Decomposition for rectangular matrices
/// Matrix A (m x n) where m >= n
pub struct SingularValueDecomp {
    pub rows: usize,
    pub cols: usize,
    pub ok: bool,
    
    // Public data
    pub a: Vec<f64>,      // Design matrix (rows x cols)
    pub w: Vec<f64>,      // Singular values (cols)
    pub v: Vec<f64>,      // Right singular vectors (cols x cols)
    pub b: Vec<f64>,      // Right-hand side (rows)
    
    // Private/internal data
    u: Option<Vec<f64>>,  // Preserved copy of 'a' if save_a is true
    work: Vec<f64>,       // Work vector (cols)
    norm: f64,            // Norm accumulator
}

impl SingularValueDecomp {
    /// Create a new SVD decomposition object
    /// 
    /// # Arguments
    /// * `rows` - Number of rows (m)
    /// * `cols` - Number of columns (n), must be <= rows
    /// * `save_a` - If true, preserve the original matrix 'a'
    pub fn new(rows: usize, cols: usize, save_a: bool) -> Option<Self> {
        if cols > rows {
            return None;
        }

        let a = vec![0.0; rows * cols];
        let w = vec![0.0; cols];
        let v = vec![0.0; cols * cols];
        let b = vec![0.0; rows];
        let work = vec![0.0; cols];
        let u = if save_a {
            Some(vec![0.0; rows * cols])
        } else {
            None
        };

        Some(SingularValueDecomp {
            rows,
            cols,
            ok: true,
            a,
            w,
            v,
            b,
            u,
            work,
            norm: 0.0,
        })
    }

    /// Helper function: compute sqrt(x^2 + y^2) avoiding overflow/underflow
    fn root_ss(x: f64, y: f64) -> f64 {
        let x = x.abs();
        let y = y.abs();
        
        if x > y {
            let ratio = y / x;
            x * (ratio * ratio + 1.0).sqrt()
        } else if y == 0.0 {
            0.0
        } else {
            let ratio = x / y;
            y * (ratio * ratio + 1.0).sqrt()
        }
    }

    /// Perform singular value decomposition
    pub fn svdcmp(&mut self) {
        let matrix = if self.u.is_some() {
            // Copy 'a' to 'u' and work on 'u'
            let a_copy = self.a.clone();
            self.u.as_mut().unwrap().copy_from_slice(&a_copy);
            self.u.as_mut().unwrap()
        } else {
            // Work directly on 'a'
            &mut self.a
        };

        self.bidiag(matrix);
        self.right(matrix);
        self.left(matrix);

        let mut sval = self.cols;
        while sval > 0 {
            sval -= 1;
            let mut iter_limit = 50;
            
            while iter_limit > 0 {
                iter_limit -= 1;
                
                let mut split = sval + 1;
                while split > 0 {
                    split -= 1;
                    
                    if self.norm + self.work[split].abs() == self.norm {
                        break;
                    }
                    if split > 0 && self.norm + self.w[split - 1].abs() == self.norm {
                        self.cancel(split, sval, matrix);
                        break;
                    }
                }
                
                if split == sval {
                    // Converged
                    if self.w[sval] < 0.0 {
                        self.w[sval] = -self.w[sval];
                        for i in 0..self.cols {
                            self.v[i * self.cols + sval] = -self.v[i * self.cols + sval];
                        }
                    }
                    break;
                }
                
                self.qr(split, sval, matrix);
            }
        }
    }

    /// Householder reduction to bidiagonal
    fn bidiag(&mut self, matrix: &mut [f64]) {
        self.norm = 0.0;
        let mut temp = 0.0;
        let mut scale;

        for col in 0..self.cols {
            self.work[col] = scale * temp;

            scale = 0.0;
            for k in col..self.rows {
                scale += matrix[k * self.cols + col].abs();
            }

            if scale > 0.0 {
                self.w[col] = scale * self.bid1(col, matrix, scale);
            } else {
                self.w[col] = 0.0;
            }

            scale = 0.0;
            for k in (col + 1)..self.cols {
                scale += matrix[col * self.cols + k].abs();
            }

            if scale > 0.0 {
                temp = self.bid2(col, matrix, scale);
            } else {
                temp = 0.0;
            }

            let testnorm = self.w[col].abs() + self.work[col].abs();
            if testnorm > self.norm {
                self.norm = testnorm;
            }
        }
    }

    fn bid1(&mut self, col: usize, matrix: &mut [f64], scale: f64) -> f64 {
        let mut sum = 0.0;
        for i in col..self.rows {
            let fac = matrix[i * self.cols + col] / scale;
            matrix[i * self.cols + col] = fac;
            sum += fac * fac;
        }
        
        let mut rv = sum.sqrt();
        let diag = matrix[col * self.cols + col];
        
        if diag > 0.0 {
            rv = -rv;
        }
        
        let fac = 1.0 / (diag * rv - sum);
        matrix[col * self.cols + col] = diag - rv;

        for j in (col + 1)..self.cols {
            sum = 0.0;
            for i in col..self.rows {
                sum += matrix[i * self.cols + col] * matrix[i * self.cols + j];
            }
            sum *= fac;
            for i in col..self.rows {
                matrix[i * self.cols + j] += sum * matrix[i * self.cols + col];
            }
        }

        for i in col..self.rows {
            matrix[i * self.cols + col] *= scale;
        }

        rv
    }

    fn bid2(&mut self, col: usize, matrix: &mut [f64], scale: f64) -> f64 {
        let mut sum = 0.0;
        for i in (col + 1)..self.cols {
            let fac = matrix[col * self.cols + i] / scale;
            matrix[col * self.cols + i] = fac;
            sum += fac * fac;
        }

        let mut rv = sum.sqrt();
        let diag = matrix[col * self.cols + col + 1];
        
        if diag > 0.0 {
            rv = -rv;
        }

        matrix[col * self.cols + col + 1] = diag - rv;
        let fac = 1.0 / (diag * rv - sum);
        
        for i in (col + 1)..self.cols {
            self.work[i] = fac * matrix[col * self.cols + i];
        }

        for j in (col + 1)..self.rows {
            sum = 0.0;
            for i in (col + 1)..self.cols {
                sum += matrix[j * self.cols + i] * matrix[col * self.cols + i];
            }
            for i in (col + 1)..self.cols {
                matrix[j * self.cols + i] += sum * self.work[i];
            }
        }
        
        for i in (col + 1)..self.cols {
            matrix[col * self.cols + i] *= scale;
        }
        
        rv
    }

    /// Accumulate right transforms
    fn right(&mut self, matrix: &[f64]) {
        let mut denom = 0.0;
        let mut col = self.cols;
        
        while col > 0 {
            col -= 1;
            
            if denom != 0.0 {
                let temp = 1.0 / matrix[col * self.cols + col + 1];
                
                for i in (col + 1)..self.cols {
                    self.v[i * self.cols + col] = temp * matrix[col * self.cols + i] / denom;
                }
                
                for i in (col + 1)..self.cols {
                    let mut sum = 0.0;
                    for j in (col + 1)..self.cols {
                        sum += self.v[j * self.cols + i] * matrix[col * self.cols + j];
                    }
                    for j in (col + 1)..self.cols {
                        self.v[j * self.cols + i] += sum * self.v[j * self.cols + col];
                    }
                }
            }

            denom = self.work[col];

            for i in (col + 1)..self.cols {
                self.v[col * self.cols + i] = 0.0;
                self.v[i * self.cols + col] = 0.0;
            }
            self.v[col * self.cols + col] = 1.0;
        }
    }

    /// Accumulate left transforms
    fn left(&mut self, matrix: &mut [f64]) {
        let mut col = self.cols;
        
        while col > 0 {
            col -= 1;

            for i in (col + 1)..self.cols {
                matrix[col * self.cols + i] = 0.0;
            }

            if self.w[col] == 0.0 {
                for i in col..self.rows {
                    matrix[i * self.cols + col] = 0.0;
                }
            } else {
                let fac = 1.0 / self.w[col];
                let temp = fac / matrix[col * self.cols + col];

                for i in (col + 1)..self.cols {
                    let mut sum = 0.0;
                    for j in (col + 1)..self.rows {
                        sum += matrix[j * self.cols + col] * matrix[j * self.cols + i];
                    }
                    sum *= temp;
                    for j in col..self.rows {
                        matrix[j * self.cols + i] += sum * matrix[j * self.cols + col];
                    }
                }
                
                for i in col..self.rows {
                    matrix[i * self.cols + col] *= fac;
                }
            }

            matrix[col * self.cols + col] += 1.0;
        }
    }

    /// Cancel phase
    fn cancel(&mut self, low: usize, high: usize, matrix: &mut [f64]) {
        let lm1 = low - 1;
        let mut sine = 1.0;
        
        for col in low..=high {
            let leg1 = sine * self.work[col];
            
            if self.norm + leg1.abs() != self.norm {
                let leg2 = self.w[col];
                let svhypot = Self::root_ss(leg1, leg2);
                self.w[col] = svhypot;
                sine = -leg1 / svhypot;
                let cosine = leg2 / svhypot;
                
                for row in 0..self.rows {
                    let x = matrix[row * self.cols + col];
                    let y = matrix[row * self.cols + lm1];
                    matrix[row * self.cols + col] = x * cosine - y * sine;
                    matrix[row * self.cols + lm1] = x * sine + y * cosine;
                }
            }
        }
    }

    /// QR decomposition phase
    fn qr(&mut self, low: usize, high: usize, matrix: &mut [f64]) {
        let wh = self.w[high];
        let whm1 = self.w[high - 1];
        let wkh = self.work[high];
        let wkhm1 = self.work[high - 1];
        
        let temp = 2.0 * wkh * whm1;
        let temp = if temp != 0.0 {
            ((whm1 + wh) * (whm1 - wh) + (wkhm1 + wkh) * (wkhm1 - wkh)) / temp
        } else {
            0.0
        };

        let mut svhypot = Self::root_ss(temp, 1.0);
        if temp < 0.0 {
            svhypot = -svhypot;
        }

        let ww = self.w[low];
        let mut wk = wkh * (whm1 / (temp + svhypot) - wkh) + (ww + wh) * (ww - wh);
        if ww != 0.0 {
            wk /= ww;
        } else {
            wk = 0.0;
        }

        let mut sine = 1.0;
        let mut cosine = 1.0;

        for col in low..high {
            let x = self.work[col + 1];
            let ty = sine * x;
            let mut x = x * cosine;
            svhypot = Self::root_ss(wk, ty);
            self.work[col] = svhypot;
            cosine = wk / svhypot;
            sine = ty / svhypot;
            
            let tx = ww * cosine + x * sine;
            x = x * cosine - ww * sine;
            
            let y = self.w[col + 1];
            let ty = y * sine;
            let y = y * cosine;
            
            self.qr_vrot(col, sine, cosine);
            svhypot = Self::root_ss(tx, ty);
            self.w[col] = svhypot;
            
            if svhypot != 0.0 {
                cosine = tx / svhypot;
                sine = ty / svhypot;
            }
            
            self.qr_mrot(col, sine, cosine, matrix);
            wk = cosine * x + sine * y;
            let ww_new = cosine * y - sine * x;
            
            // Update ww for next iteration (it's not used after loop but for consistency)
            let _ = ww_new;
        }
        
        self.work[low] = 0.0;
        self.work[high] = wk;
    }

    /// Right vector rotation for QR
    fn qr_vrot(&mut self, col: usize, sine: f64, cosine: f64) {
        for row in 0..self.cols {
            let vptr = row * self.cols + col;
            let x = self.v[vptr];
            let y = self.v[vptr + 1];
            self.v[vptr] = x * cosine + y * sine;
            self.v[vptr + 1] = y * cosine - x * sine;
        }
    }

    /// Matrix rotation for QR
    fn qr_mrot(&mut self, col: usize, sine: f64, cosine: f64, matrix: &mut [f64]) {
        for row in 0..self.rows {
            let mptr = row * self.cols + col;
            let x = matrix[mptr];
            let y = matrix[mptr + 1];
            matrix[mptr] = x * cosine + y * sine;
            matrix[mptr + 1] = y * cosine - x * sine;
        }
    }

    /// Back-substitution to solve Ax = b
    /// 
    /// # Arguments
    /// * `limit` - Singular value threshold (relative to max singular value)
    /// * `soln` - Output: solution vector
    pub fn backsub(&self, limit: f64, soln: &mut [f64]) {
        let matrix = if self.u.is_some() {
            self.u.as_ref().unwrap()
        } else {
            &self.a
        };

        // Find maximum singular value
        let mut wmax = 0.0;
        for i in 0..self.cols {
            if i == 0 || self.w[i] > wmax {
                wmax = self.w[i];
            }
        }

        let limit = limit * wmax + EPSILON;

        // Find U'b
        let mut utb = vec![0.0; self.cols];
        for i in 0..self.cols {
            let mut sum = 0.0;
            if self.w[i] > limit {
                for j in 0..self.rows {
                    sum += matrix[j * self.cols + i] * self.b[j];
                }
                sum /= self.w[i];
            }
            utb[i] = sum;
        }

        // Multiply by V to complete the solution
        for i in 0..self.cols {
            let mut sum = 0.0;
            for j in 0..self.cols {
                sum += self.v[i * self.cols + j] * utb[j];
            }
            soln[i] = sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svd_creation() {
        let svd = SingularValueDecomp::new(5, 3, false);
        assert!(svd.is_some());
        let svd = svd.unwrap();
        assert_eq!(svd.rows, 5);
        assert_eq!(svd.cols, 3);
        assert!(svd.ok);
    }

    #[test]
    fn test_svd_invalid_dimensions() {
        let svd = SingularValueDecomp::new(3, 5, false);
        assert!(svd.is_none());
    }

    #[test]
    fn test_svd_with_save() {
        let svd = SingularValueDecomp::new(4, 3, true);
        assert!(svd.is_some());
        let svd = svd.unwrap();
        assert!(svd.u.is_some());
    }

    #[test]
    fn test_simple_decomposition() {
        let mut svd = SingularValueDecomp::new(3, 2, true).unwrap();
        
        // Create a simple 3x2 matrix
        // [1 0]
        // [0 1]
        // [0 0]
        svd.a[0] = 1.0;
        svd.a[2] = 1.0;
        
        svd.svdcmp();
        
        // Check that we got singular values
        assert!(svd.w[0] > 0.0);
        assert!(svd.w[1] > 0.0);
    }

    #[test]
    fn test_root_ss() {
        let result = SingularValueDecomp::root_ss(3.0, 4.0);
        assert!((result - 5.0).abs() < 1e-10);
        
        let result = SingularValueDecomp::root_ss(-3.0, -4.0);
        assert!((result - 5.0).abs() < 1e-10);
    }
}

fn main() {
    println!("Singular Value Decomposition library for Rust");
    
    // Example usage
    let mut svd = SingularValueDecomp::new(5, 3, true)
        .expect("Failed to create SVD object");
    
    // Set up a simple matrix (5x3)
    for i in 0..5 {
        for j in 0..3 {
            svd.a[i * 3 + j] = (i * 3 + j + 1) as f64;
        }
    }
    
    println!("Original matrix (5x3):");
    for i in 0..5 {
        for j in 0..3 {
            print!("{:8.2} ", svd.a[i * 3 + j]);
        }
        println!();
    }
    
    svd.svdcmp();
    
    println!("\nSingular values:");
    for i in 0..3 {
        println!("  w[{}] = {:.6}", i, svd.w[i]);
    }
    
    // Set up a right-hand side for back-substitution
    for i in 0..5 {
        svd.b[i] = (i + 1) as f64;
    }
    
    let mut solution = vec![0.0; 3];
    svd.backsub(1e-8, &mut solution);
    
    println!("\nSolution from back-substitution:");
    for i in 0..3 {
        println!("  x[{}] = {:.6}", i, solution[i]);
    }
}