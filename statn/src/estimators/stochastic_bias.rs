
//// Stochastic Bias Estimator
/// 
/// Estimates training bias from data collected from a stochastic training procedure.
/// The key idea is to track in-sample (IS) best performance and out-of-sample (OOS)
/// performance to estimate the bias introduced by the stochastic search process.
#[derive(Clone)]
pub struct StocBias {
    nreturns: usize,
    ok: bool,
    collecting: bool,
    got_first_case: bool,
    
    // Public accessible data
    is_best: Vec<f64>,
    oos: Vec<f64>,
    returns: Vec<f64>,
}

impl StocBias {
    /// Create a new StocBias estimator
    ///
    /// # Arguments
    /// * `nc` - Number of returns to track
    ///
    /// # Returns
    /// `Some(StocBias)` if successful, `None` if memory allocation fails
    pub fn new(nc: usize) -> Option<Self> {
        if nc == 0 {
            return None;
        }

        let is_best = vec![0.0; nc];
        let oos = vec![0.0; nc];
        let returns = vec![0.0; nc];

        Some(StocBias {
            nreturns: nc,
            ok: true,
            collecting: false,
            got_first_case: false,
            is_best,
            oos,
            returns,
        })
    }

    /// Check if the object was successfully initialized
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Get the number of returns being tracked
    pub fn num_returns(&self) -> usize {
        self.nreturns
    }

    /// Enable or disable data collection
    ///
    /// We must collect only during random or exhaustive search, never
    /// during an intelligent, guided search.
    ///
    /// # Arguments
    /// * `collect_data` - true to enable collection, false to disable
    pub fn set_collecting(&mut self, collect_data: bool) {
        self.collecting = collect_data;
    }

    /// Check if currently collecting data
    pub fn is_collecting(&self) -> bool {
        self.collecting
    }

    /// Get a mutable reference to the returns vector
    ///
    /// This is called by the criterion routine to provide a place
    /// to store bar returns.
    pub fn returns_mut(&mut self) -> &mut [f64] {
        &mut self.returns
    }

    /// Get an immutable reference to the returns vector
    pub fn returns(&self) -> &[f64] {
        &self.returns
    }

    /// Get a reference to the IS best vector
    pub fn is_best(&self) -> &[f64] {
        &self.is_best
    }

    /// Get a reference to the OOS vector
    pub fn oos(&self) -> &[f64] {
        &self.oos
    }

    /// Process the current set of returns
    ///
    /// This function maintains the best in-sample performance and corresponding
    /// out-of-sample performance. The idea is that for each case we exclude it
    /// from the in-sample data and compute the performance of the remaining data.
    /// The excluded case becomes the out-of-sample data.
    pub fn process(&mut self) {
        if !self.collecting {
            return;
        }

        // Compute total return
        let total: f64 = self.returns.iter().sum();

        // Initialize if this is the first call
        if !self.got_first_case {
            self.got_first_case = true;
            for i in 0..self.nreturns {
                let this_x = self.returns[i];
                self.is_best[i] = total - this_x;
                self.oos[i] = this_x;
            }
        }
        // Keep track of best if this is a subsequent call
        else {
            for i in 0..self.nreturns {
                let this_x = self.returns[i];
                let is_candidate = total - this_x;
                if is_candidate > self.is_best[i] {
                    self.is_best[i] = is_candidate;
                    self.oos[i] = this_x;
                }
            }
        }
    }

    /// Compute final bias statistics
    ///
    /// This computes the final in-sample return, out-of-sample return, and bias.
    /// The normal situation will be for the supplied returns to be log bar returns.
    /// This works on the basis of total log return.
    ///
    /// # Returns
    /// A tuple of (is_return, oos_return, bias)
    pub fn compute(&self) -> (f64, f64, f64) {
        let mut is_return = 0.0;
        let mut oos_return = 0.0;

        for i in 0..self.nreturns {
            is_return += self.is_best[i];
            oos_return += self.oos[i];
        }

        // Each IS_best is the sum of nreturns-1 returns
        is_return /= (self.nreturns - 1) as f64;
        let bias = is_return - oos_return;

        (is_return, oos_return, bias)
    }

    /// Reset the state (useful for repeated use)
    pub fn reset(&mut self) {
        self.is_best.fill(0.0);
        self.oos.fill(0.0);
        self.returns.fill(0.0);
        self.got_first_case = false;
        self.collecting = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let sb = StocBias::new(10);
        assert!(sb.is_some());
        let sb = sb.unwrap();
        assert!(sb.is_ok());
        assert_eq!(sb.num_returns(), 10);
    }

    #[test]
    fn test_creation_zero() {
        let sb = StocBias::new(0);
        assert!(sb.is_none());
    }

    #[test]
    fn test_collecting_toggle() {
        let mut sb = StocBias::new(5).unwrap();
        assert!(!sb.is_collecting());
        
        sb.set_collecting(true);
        assert!(sb.is_collecting());
        
        sb.set_collecting(false);
        assert!(!sb.is_collecting());
    }

    #[test]
    fn test_no_process_when_not_collecting() {
        let mut sb = StocBias::new(3).unwrap();
        sb.set_collecting(false);
        
        // Set some returns
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        
        sb.process();
        
        // Nothing should have been processed
        assert_eq!(sb.is_best()[0], 0.0);
        assert_eq!(sb.oos()[0], 0.0);
    }

    #[test]
    fn test_simple_processing() {
        let mut sb = StocBias::new(3).unwrap();
        sb.set_collecting(true);
        
        // Set some returns
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        
        sb.process();
        
        // Total = 6.0
        // IS_best should be [5.0, 4.0, 3.0] (total - each)
        // OOS should be [1.0, 2.0, 3.0] (each return)
        assert_eq!(sb.is_best()[0], 5.0);
        assert_eq!(sb.is_best()[1], 4.0);
        assert_eq!(sb.is_best()[2], 3.0);
        assert_eq!(sb.oos()[0], 1.0);
        assert_eq!(sb.oos()[1], 2.0);
        assert_eq!(sb.oos()[2], 3.0);
    }

    #[test]
    fn test_improvement_tracking() {
        let mut sb = StocBias::new(3).unwrap();
        sb.set_collecting(true);
        
        // First round
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        sb.process();
        
        // Second round with better values
        let returns = sb.returns_mut();
        returns[0] = 2.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        sb.process();
        
        // After second round:
        // Total = 7.0
        // For i=0: 7-2=5 > 5? No, stays at 5.0
        // For i=1: 7-2=5 > 4? Yes, becomes 5.0
        // For i=2: 7-3=4 > 3? Yes, becomes 4.0
        assert_eq!(sb.is_best()[0], 5.0);
        assert_eq!(sb.is_best()[1], 5.0);
        assert_eq!(sb.is_best()[2], 4.0);
        assert_eq!(sb.oos()[0], 1.0);
        assert_eq!(sb.oos()[1], 2.0);
        assert_eq!(sb.oos()[2], 3.0);
    }

    #[test]
    fn test_compute_bias() {
        let mut sb = StocBias::new(3).unwrap();
        sb.set_collecting(true);
        
        // Set some returns
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        
        sb.process();
        
        let (is_ret, oos_ret, bias) = sb.compute();
        
        // IS_best = [5.0, 4.0, 3.0], sum = 12.0, divided by (3-1) = 2 gives 6.0
        // OOS = [1.0, 2.0, 3.0], sum = 6.0
        // Bias = 6.0 - 6.0 = 0.0
        assert_eq!(is_ret, 6.0);
        assert_eq!(oos_ret, 6.0);
        assert_eq!(bias, 0.0);
    }

    #[test]
    fn test_compute_with_bias() {
        let mut sb = StocBias::new(2).unwrap();
        sb.set_collecting(true);
        
        // Set returns
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 5.0;
        
        sb.process();
        
        let (is_ret, oos_ret, bias) = sb.compute();
        
        // IS_best = [5.0, 1.0], sum = 6.0, divided by (2-1) = 1 gives 6.0
        // OOS = [1.0, 5.0], sum = 6.0
        // Bias = 6.0 - 6.0 = 0.0
        assert_eq!(is_ret, 6.0);
        assert_eq!(oos_ret, 6.0);
        assert_eq!(bias, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut sb = StocBias::new(3).unwrap();
        sb.set_collecting(true);
        
        // Set some returns and process
        let returns = sb.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        sb.process();
        
        assert!(sb.is_best()[0] > 0.0);
        
        // Reset
        sb.reset();
        
        assert_eq!(sb.is_best()[0], 0.0);
        assert!(!sb.is_collecting());
    }
}
