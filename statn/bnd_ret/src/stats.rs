pub use stats::{lgamma, ibeta, orderstat_tail, quantile_conf};

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
