use anyhow::Result;
use statn::models::cd_ma::{CoordinateDescent, cv_train};

/// Result of model training
pub struct TrainingResult {
    /// Trained coordinate descent model
    pub model: CoordinateDescent,
    /// Optimal lambda value
    pub lambda: f64,
    /// All tested lambda values
    pub lambdas: Vec<f64>,
    /// Out-of-sample performance for each lambda
    pub lambda_oos: Vec<f64>,
}

/// Train model with cross-validation to find optimal lambda
pub fn train_with_cv(
    n_vars: usize,
    n_cases: usize,
    data: &[f64],
    targets: &[f64],
    alpha: f64,
    n_folds: usize,
    n_lambdas: usize,
    max_iterations: usize,
    tolerance: f64,
) -> Result<TrainingResult> {
    println!("Running {}-fold cross-validation...", n_folds);
    
    let mut lambdas = vec![0.0; n_lambdas];
    let mut lambda_oos = vec![0.0; n_lambdas];
    
    let lambda = if alpha <= 0.0 {
        println!("Alpha <= 0, using lambda = 0 (no regularization)");
        0.0
    } else {
        cv_train(
            n_vars,
            n_folds,
            data,
            targets,
            None,
            &mut lambdas,
            &mut lambda_oos,
            true,  // covar_updates
            n_lambdas,
            alpha,
            max_iterations,
            tolerance,
            true,  // fast_test
        )
    };
    
    println!("Optimal lambda: {:.6}", lambda);
    
    // Train final model with optimal lambda
    println!("Training final model...");
    let mut model = CoordinateDescent::new(n_vars, n_cases, false, true, 0);
    model.get_data(0, n_cases, data, targets, None);
    model.core_train(alpha, lambda, max_iterations, 1e-7, true, false);
    
    println!("In-sample explained variance: {:.3}%", 100.0 * model.explained);
    
    Ok(TrainingResult {
        model,
        lambda,
        lambdas,
        lambda_oos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_train_with_cv_zero_alpha() {
        let n_vars = 5;
        let n_cases = 100;
        let data = vec![0.0; n_vars * n_cases];
        let targets = vec![0.0; n_cases];
        
        let result = train_with_cv(
            n_vars,
            n_cases,
            &data,
            &targets,
            0.0,  // Zero alpha
            5,
            10,
            100,
            1e-6,
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.lambda, 0.0);
    }
}
