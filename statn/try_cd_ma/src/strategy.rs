use backtesting::Strategy;
use statn::models::cd_ma::CoordinateDescent;

/// Strategy wrapper for Coordinate Descent Moving Average model
pub struct CDMAStrategy {
    /// Trained model
    model: CoordinateDescent,
    /// Pre-computed indicator data (standardized)
    /// Stored as a flat vector: [case0_var0, case0_var1, ..., case1_var0, ...]
    data: Vec<f64>,
    /// Number of variables per case
    n_vars: usize,
    /// Offset to map backtest index to data index
    /// Backtest index starts at 0 for the first test case
    /// Data index might be offset if we only computed indicators for the test set
    offset: usize,
}

impl CDMAStrategy {
    /// Create a new CDMA strategy
    pub fn new(
        model: CoordinateDescent,
        data: Vec<f64>,
        n_vars: usize,
        offset: usize,
    ) -> Self {
        Self {
            model,
            data,
            n_vars,
            offset,
        }
    }
}

impl Strategy for CDMAStrategy {
    fn signal(&self, _prices: &[f64], index: usize) -> f64 {
        // Map backtest index to data index
        // If backtest starts at index 0 of prices, and we have indicators for all prices,
        // then index corresponds to data index.
        // However, we usually only have indicators starting from max_lookback.
        // The backtester iterates from 0 to n-1.
        // If we pass the full price history to backtester, index 0 is the first price.
        // But we can't trade until max_lookback.
        // So we should return 0.0 until we have data.
        
        if index < self.offset {
            return 0.0;
        }
        
        let data_idx = index - self.offset;
        
        // Check bounds
        if (data_idx + 1) * self.n_vars > self.data.len() {
            return 0.0;
        }
        
        let xptr = &self.data[data_idx * self.n_vars..(data_idx + 1) * self.n_vars];
        
        // Compute prediction
        let pred: f64 = xptr
            .iter()
            .enumerate()
            .map(|(ivar, &x)| {
                self.model.beta[ivar] * (x - self.model.xmeans[ivar]) / self.model.xscales[ivar]
            })
            .sum();
        
        let pred = pred * self.model.yscale + self.model.ymean;
        
        // Trading logic: long if pred > 0, short if pred < 0
        if pred > 0.0 {
            1.0
        } else if pred < 0.0 {
            -1.0
        } else {
            0.0
        }
    }
}
