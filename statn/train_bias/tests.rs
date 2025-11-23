#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use crate::opt::{OptCriteria, opt_params};
    use crate::test_system::test_system;

    #[test]
    fn test_rng_seed() {
        let mut rng = Rng::new();
        rng.seed(12345);
        let val1 = rng.uniform();
        let mut rng2 = Rng::new();
        rng2.seed(12345);
        let val2 = rng2.uniform();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rng_range() {
        let mut rng = Rng::new();
        rng.seed(12345);
        for _ in 0..100 {
            let u = rng.uniform();
            assert!(u >= 0.0 && u < 1.0);
        }
    }

    #[test]
    fn test_opt_params() {
        let mut rng = Rng::new();
        rng.seed(12345);
        let mut x = vec![0.0; 500];
        x[0] = 0.0;
        for i in 1..500 {
            x[i] = x[i - 1] + rng.uniform() - 0.5;
        }
        let (perf, short, long) = opt_params(OptCriteria::MeanReturn, &x);
        assert!(short >= 1);
        assert!(long >= 2);
        assert!(long > short);
        assert!(!perf.is_nan());
    }

    #[test]
    fn test_test_system() {
        let mut x = vec![0.0; 100];
        for i in 1..100 {
            x[i] = x[i - 1] + 0.01;
        }
        let result = test_system(&x, 5, 10);
        assert!(!result.is_nan());
    }

    #[test]
    fn test_opt_criteria_from_u32() {
        assert!(matches!(OptCriteria::from_u32(0), Some(OptCriteria::MeanReturn)));
        assert!(matches!(OptCriteria::from_u32(1), Some(OptCriteria::ProfitFactor)));
        assert!(matches!(OptCriteria::from_u32(2), Some(OptCriteria::SharpeRatio)));
        assert_eq!(OptCriteria::from_u32(3), None);
    }

    #[test]
    fn test_price_generation() {
        let mut rng = Rng::new();
        rng.seed(42);
        let ncases = 100;
        let trend = 0.1;
        let mut x = vec![0.0; ncases];
        x[0] = 0.0;
        for i in 1..ncases {
            x[i] = x[i - 1] + trend + rng.uniform() - 0.5;
        }
        assert_eq!(x.len(), ncases);
        assert_eq!(x[0], 0.0);
        for i in 1..ncases {
            assert_ne!(x[i], x[i - 1]);
        }
    }
}
