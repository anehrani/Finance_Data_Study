/// Criterion function for CSCV
/// 
/// Active implementation: calculates the mean of returns
fn criter(returns: &[f64]) -> f64 {
    returns.iter().sum::<f64>() / returns.len() as f64
}

/// Alternative implementation: calculates win/loss ratio
/// Currently inactive
#[allow(dead_code)]
fn criter_alternative(returns: &[f64]) -> f64 {
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
        let returns = vec![1.0, 2.0, 3.0, 4.0];
        assert!((criter(&returns) - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_criter_alternative_ratio() {
        let returns = vec![1.0, 2.0, -0.5, -0.5];
        let result = criter_alternative(&returns);
        assert!(result > 0.0);
    }
}