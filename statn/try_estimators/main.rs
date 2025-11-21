
use statn::estimators::StocBias;

fn main() {
    println!("Stochastic Bias Estimator\n");
    
    // Create a bias estimator for 5 returns
    let mut bias_est = StocBias::new(5).expect("Failed to create StocBias");
    
    println!("Created StocBias with {} returns", bias_est.num_returns());
    println!("Object initialized: {}\n", bias_est.is_ok());
    
    // Enable data collection
    bias_est.set_collecting(true);
    println!("Collecting enabled: {}\n", bias_est.is_collecting());
    
    // Simulate first round of returns
    println!("Round 1: Processing returns [1.0, 2.0, 3.0, 4.0, 5.0]");
    {
        let returns = bias_est.returns_mut();
        returns[0] = 1.0;
        returns[1] = 2.0;
        returns[2] = 3.0;
        returns[3] = 4.0;
        returns[4] = 5.0;
    }
    bias_est.process();
    
    println!("IS_best: {:?}", bias_est.is_best());
    println!("OOS:     {:?}\n", bias_est.oos());
    
    // Simulate second round with different returns
    println!("Round 2: Processing returns [1.5, 2.5, 3.5, 4.5, 5.5]");
    {
        let returns = bias_est.returns_mut();
        returns[0] = 1.5;
        returns[1] = 2.5;
        returns[2] = 3.5;
        returns[3] = 4.5;
        returns[4] = 5.5;
    }
    bias_est.process();
    
    println!("IS_best: {:?}", bias_est.is_best());
    println!("OOS:     {:?}\n", bias_est.oos());
    
    // Compute final bias
    let (is_return, oos_return, bias) = bias_est.compute();
    
    println!("Final Results:");
    println!("  In-Sample Return:  {:.6}", is_return);
    println!("  Out-of-Sample Return: {:.6}", oos_return);
    println!("  Bias Estimate:     {:.6}", bias);
}