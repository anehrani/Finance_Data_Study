use statn::estimators::sensitivity::sensitivity;

/// Example criterion function: Sphere function
/// Optimal at params = [0, 0, 0], maximum value = 0
fn sphere_function(params: &[f64], _mintrades: i32) -> f64 {
    let mut sum = 0.0;
    for &x in params {
        sum += x * x;
    }
    -sum // Negative because we're maximizing
}

/// Example criterion function: Rosenbrock function (modified for maximization)
/// Optimal at params = [1, 1], maximum value = 0
fn rosenbrock_function(params: &[f64], _mintrades: i32) -> f64 {
    if params.len() < 2 {
        return -1e10;
    }
    let mut sum = 0.0;
    for i in 0..params.len() - 1 {
        let term1 = params[i + 1] - params[i] * params[i];
        let term2 = 1.0 - params[i];
        sum += 100.0 * term1 * term1 + term2 * term2;
    }
    -sum // Negative because we're maximizing
}

/// Example criterion function: Simple quadratic
fn quadratic_function(params: &[f64], _mintrades: i32) -> f64 {
    let mut result = 0.0;
    for (i, &p) in params.iter().enumerate() {
        let optimal = (i + 1) as f64 * 2.0;
        let diff = p - optimal;
        result += 10.0 - diff * diff;
    }
    result
}

fn main() {
    println!("Parameter Sensitivity Analysis Tool\n");
    println!("This tool demonstrates the sensitivity analysis function");
    println!("from statn::estimators::sensitivity\n");

    // Example 1: Sphere function with 3 real parameters
    println!("=== Example 1: Sphere Function (3 real parameters) ===");
    let nvars = 3;
    let nints = 0;
    let best = vec![0.0, 0.0, 0.0]; // Optimal point
    let low_bounds = vec![-5.0, -5.0, -5.0];
    let high_bounds = vec![5.0, 5.0, 5.0];

    match sensitivity(
        sphere_function,
        nvars,
        nints,
        20,  // npoints
        60,  // nres (histogram width)
        10,  // mintrades (not used in this example)
        &best,
        &low_bounds,
        &high_bounds,
    ) {
        Ok(_) => println!("✓ Sensitivity analysis completed. Results saved to SENS.LOG\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 2: Quadratic function with mixed integer/real parameters
    println!("=== Example 2: Quadratic Function (1 integer + 2 real parameters) ===");
    let nvars = 3;
    let nints = 1; // First parameter is integer
    let best = vec![2.0, 4.0, 6.0]; // Optimal point
    let low_bounds = vec![0.0, 0.0, 0.0];
    let high_bounds = vec![10.0, 10.0, 10.0];

    match sensitivity(
        quadratic_function,
        nvars,
        nints,
        15,  // npoints
        50,  // nres
        10,  // mintrades
        &best,
        &low_bounds,
        &high_bounds,
    ) {
        Ok(_) => println!("✓ Sensitivity analysis completed. Results saved to SENS.LOG\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 3: Rosenbrock function
    println!("=== Example 3: Rosenbrock Function (2 real parameters) ===");
    let nvars = 2;
    let nints = 0;
    let best = vec![1.0, 1.0]; // Optimal point
    let low_bounds = vec![-2.0, -2.0];
    let high_bounds = vec![3.0, 3.0];

    match sensitivity(
        rosenbrock_function,
        nvars,
        nints,
        25,  // npoints
        70,  // nres
        10,  // mintrades
        &best,
        &low_bounds,
        &high_bounds,
    ) {
        Ok(_) => println!("✓ Sensitivity analysis completed. Results saved to SENS.LOG\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }


    println!("All examples completed!");
    println!("Check SENS.LOG for detailed sensitivity curves.");
}