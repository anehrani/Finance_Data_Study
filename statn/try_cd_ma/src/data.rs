// Re-export from shared I/O modules
pub use statn::core::io::{
    read_price_file as load_prices,
    split_train_test,
    compute_targets,
    DataSplit,
};

/// Market data structure
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Log prices
    pub prices: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_prices() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0").unwrap();
        writeln!(file, "20200102 101.5").unwrap();
        writeln!(file, "20200103 99.8").unwrap();
        
        let prices = load_prices(file.path()).unwrap();
        assert_eq!(prices.len(), 3);
        assert!((prices[0] - 100.0_f64.ln()).abs() < 1e-10);
    }
    
    #[test]
    fn test_split_train_test() {
        let prices: Vec<f64> = (0..1000).map(|i| (100.0 + i as f64).ln()).collect();
        let split = split_train_test(&prices, 200, 252).unwrap();
        
        assert_eq!(split.max_lookback, 200);
        assert!(split.train_data.len() > 0);
        assert!(split.test_data.len() >= 252 + 200);
    }
    
    #[test]
    fn test_compute_targets() {
        let prices = vec![1.0, 1.1, 1.05, 1.15];
        let targets = compute_targets(&prices, 0, 3);
        
        assert_eq!(targets.len(), 3);
        assert!((targets[0] - 0.1).abs() < 1e-10);
        assert!((targets[1] - (-0.05)).abs() < 1e-10);
        assert!((targets[2] - 0.1).abs() < 1e-10);
    }
}
