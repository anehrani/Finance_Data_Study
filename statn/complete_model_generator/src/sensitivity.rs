use statn::estimators::sensitivity::sensitivity;
use try_cd_ma::run_backtest;
use try_cd_ma::Config;

pub fn run_sensitivity_analysis(
    _config: &Config,
    best_n_long: usize,
    best_n_short: usize,
    best_alpha: f64,
    _market_data_path: &str,
    output_path: &std::path::Path,
) -> Result<(), String> {
    println!("\n=== Running Sensitivity Analysis ===");

    // Define the criterion function
    // Params: [n_long, n_short, alpha]
    let criterion = |params: &[f64], _mintrades: i32| -> f64 {
        let n_long = params[0].round() as usize;
        let n_short = params[1].round() as usize;
        let alpha = params[2];

        if n_long <= n_short || n_short < 2 || alpha <= 0.0 || alpha > 1.0 {
            return -1e10;
        }

        // Placeholder return
        0.0 
    };

    // Setup sensitivity params
    let nvars = 3;
    let nints = 2;
    let best = vec![best_n_long as f64, best_n_short as f64, best_alpha];
    let low_bounds = vec![
        (best_n_long as f64 * 0.8).max(2.0),
        (best_n_short as f64 * 0.8).max(2.0),
        (best_alpha * 0.8).max(0.01),
    ];
    let high_bounds = vec![
        (best_n_long as f64 * 1.2),
        (best_n_short as f64 * 1.2),
        (best_alpha * 1.2).min(1.0),
    ];

    match sensitivity(
        criterion,
        nvars,
        nints,
        20,
        50,
        10,
        &best,
        &low_bounds,
        &high_bounds,
        output_path,
    ) {
        Ok(_) => {
            println!("Sensitivity analysis completed. Results in {:?}", output_path);
            Ok(())
        }
        Err(e) => Err(format!("Sensitivity analysis failed: {}", e)),
    }
}
