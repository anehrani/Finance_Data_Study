use crate::io::MarketData;
use crate::test_system::test_system;
use crate::test_system_enhanced::test_system_enhanced;
use statn::estimators::StocBias;

/// Criterion function for optimization
pub fn criter(
    params: &[f64],
    mintrades: i32,
    data: &MarketData,
    stoc_bias: &mut Option<&mut StocBias>,
) -> f64 {
    let long_term = (params[0] + 1.0e-10) as usize;
    let short_pct = params[1];
    let short_thresh = params[2];
    let long_thresh = params[3];

    let (ret_val, ntrades) = if let Some(sb) = stoc_bias {
        let returns = sb.returns_mut();
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            Some(returns),
        )
    } else {
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            None,
        )
    };

    if let Some(sb) = stoc_bias
        && ret_val > 0.0 {
            sb.process();
        }

    if ntrades >= mintrades {
        ret_val
    } else {
        -1.0e20
    }
}

/// Criterion function for optimization (Enhanced Version)
pub fn criter_enhanced(
    params: &[f64],
    mintrades: i32,
    data: &MarketData,
    stoc_bias: &mut Option<&mut StocBias>,
) -> f64 {
    let long_term = (params[0] + 1.0e-10) as usize;
    let short_pct = params[1];
    let short_thresh = params[2];
    let long_thresh = params[3];

    let (ret_val, ntrades) = if let Some(sb) = stoc_bias {
        let returns = sb.returns_mut();
        test_system_enhanced(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            Some(returns),
        )
    } else {
        test_system_enhanced(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            None,
        )
    };

    if let Some(sb) = stoc_bias
        && ret_val > 0.0 {
            sb.process();
        }

    if ntrades >= mintrades {
        ret_val
    } else {
        -1.0e20
    }
}
