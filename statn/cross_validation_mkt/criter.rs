/// Criterion function for CSCV
/// 
/// Computes the mean return from a slice of returns.
/// This is the active version from CRITER.CPP (when #if 1 is true).
/// 
/// # Arguments
/// * `returns` - Slice of return values
/// 
/// # Returns
/// Mean of the returns
pub fn criter(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = returns.iter().sum();
    sum / returns.len() as f64
}

/// Alternative criterion function (win/loss ratio)
/// This is the commented-out version from CRITER.CPP (when #if 0)
#[allow(dead_code)]
pub fn criter_win_loss_ratio(returns: &[f64]) -> f64 {
    let mut win_sum = 1.0e-60;
    let mut lose_sum = 1.0e-60;
    
    for &ret in returns {
        if ret > 0.0 {
            win_sum += ret;
        } else {
            lose_sum -= ret;
        }
    }
    
    win_sum / lose_sum
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_criter_mean() {
        let returns = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = criter(&returns);
        assert!((result - 3.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_criter_empty() {
        let returns: Vec<f64> = vec![];
        let result = criter(&returns);
        assert_eq!(result, 0.0);
    }
    
    #[test]
    fn test_criter_win_loss() {
        let returns = vec![1.0, -1.0, 2.0, -2.0];
        let result = criter_win_loss_ratio(&returns);
        // win_sum = 1e-60 + 1.0 + 2.0 = 3.0 (approx)
        // lose_sum = 1e-60 + 1.0 + 2.0 = 3.0 (approx)
        // ratio should be close to 1.0
        assert!((result - 1.0).abs() < 0.01);
    }
}
