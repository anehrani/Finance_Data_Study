/// Performance criterion types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriterionType {
    TotalReturn = 0,
    SharpeRatio = 1,
    ProfitFactor = 2,
}

impl CriterionType {
    pub fn name(&self) -> &'static str {
        match self {
            CriterionType::TotalReturn => "Total return",
            CriterionType::SharpeRatio => "Sharpe ratio",
            CriterionType::ProfitFactor => "Profit factor",
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(CriterionType::TotalReturn),
            1 => Some(CriterionType::SharpeRatio),
            2 => Some(CriterionType::ProfitFactor),
            _ => None,
        }
    }
}

/// Compute total return (assumes prices are log prices)
pub fn total_return(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices[prices.len() - 1] - prices[0]
}

/// Compute raw Sharpe ratio (assumes prices are log prices)
pub fn sharpe_ratio(prices: &[f64]) -> f64 {
    let n = prices.len();
    if n < 2 {
        return 0.0;
    }

    let mean = (prices[n - 1] - prices[0]) / (n - 1) as f64;

    let mut var = 1.0e-60; // Ensure no division by 0
    for i in 1..n {
        let diff = (prices[i] - prices[i - 1]) - mean;
        var += diff * diff;
    }

    mean / (var / (n - 1) as f64).sqrt()
}

/// Compute profit factor (assumes prices are log prices)
pub fn profit_factor(prices: &[f64]) -> f64 {
    let n = prices.len();
    if n < 2 {
        return 1.0;
    }

    let mut win_sum = 1.0e-60;
    let mut lose_sum = 1.0e-60;

    for i in 1..n {
        let ret = prices[i] - prices[i - 1];
        if ret > 0.0 {
            win_sum += ret;
        } else {
            lose_sum -= ret;
        }
    }

    win_sum / lose_sum
}

/// Master criterion function
pub fn criterion(which: CriterionType, prices: &[f64]) -> f64 {
    match which {
        CriterionType::TotalReturn => total_return(prices),
        CriterionType::SharpeRatio => sharpe_ratio(prices),
        CriterionType::ProfitFactor => profit_factor(prices),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_return() {
        let prices = vec![0.0, 0.1, 0.2, 0.15, 0.3];
        let ret = total_return(&prices);
        assert!((ret - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_sharpe_ratio() {
        let prices = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let sharpe = sharpe_ratio(&prices);
        assert!(sharpe > 0.0);
    }

    #[test]
    fn test_profit_factor() {
        let prices = vec![0.0, 0.01, -0.005, 0.015, 0.02];
        let pf = profit_factor(&prices);
        assert!(pf > 1.0);
    }
}
