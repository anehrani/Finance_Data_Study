#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use crate::opt::{OptCriteria, opt_params};
    use crate::test_system::test_system;

    #[test]
    fn test_rng_determinism() {
        let mut rng = Rng::with_seed(12345);
        let val1 = rng.unifrand();

        let mut rng2 = Rng::with_seed(12345);
        let val2 = rng2.unifrand();

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rng_distribution() {
        let mut rng = Rng::with_seed(12345);
        let mut sum = 0.0;
        let n = 10000;

        for _ in 0..n {
            let u = rng.unifrand();
            assert!(u >= 0.0 && u < 1.0);
            sum += u;
        }

        let mean = sum / n as f64;
        assert!((mean - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_opt_params_simple() {
        let mut x = vec![0.0; 100];
        let mut rng = Rng::with_seed(12345);
        
        // Create a simple uptrend
        for i in 1..100 {
            x[i] = x[i - 1] + rng.unifrand() - 0.5 + 0.1; // +0.1 bias
        }

        // Should prefer long-only
        let (perf, short, long) = opt_params(OptCriteria::MeanReturn, true, &x);
        assert!(perf > 0.0);
        assert!(short > 0);
        assert!(long > short);
    }

    #[test]
    fn test_test_system() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        // Simple uptrend. 
        // short_term=1, long_term=2.
        // i=1: s=2, l=(1+2)/2=1.5. s>l -> Buy.
        // i=2: s=3, l=(2+3)/2=2.5. s>l -> Buy.
        // ...
        // Should have positive return.
        let result = test_system(true, &x, 1, 2);
        assert!(result > 0.0);
    }

    #[test]
    fn test_opt_criteria_from_u32() {
        assert!(matches!(OptCriteria::from_u32(0), Some(OptCriteria::MeanReturn)));
        assert!(matches!(OptCriteria::from_u32(1), Some(OptCriteria::ProfitFactor)));
        assert!(matches!(OptCriteria::from_u32(2), Some(OptCriteria::SharpeRatio)));
        assert_eq!(OptCriteria::from_u32(3), None);
    }

    #[test]
    fn test_integration_run() {
        let mut x = vec![0.0; 100];
        let mut rng = Rng::with_seed(42);
        let trend = 0.01;
        
        for i in 1..100 {
            x[i] = x[i - 1] + trend + rng.unifrand() - 0.5;
        }
        
        let (perf, _, _) = opt_params(OptCriteria::MeanReturn, true, &x);
        assert!(perf > -10.0);
    }
}
