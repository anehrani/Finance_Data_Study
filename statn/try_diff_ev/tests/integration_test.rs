//! Integration tests for backtesting with common library

use try_diff_ev::{backtest_signals, generate_signals, SignalResult};

#[test]
fn test_backtest_integration() {
    // Create simple test data (log prices)
    let prices: Vec<f64> = (0..100)
        .map(|i| (100.0 + (i as f64 * 0.1).sin() * 5.0).ln())
        .collect();
    
    // Generate signals using the original generator
    let result = generate_signals(
        "original",
        &prices,
        20,    // long_lookback
        50.0,  // short_pct
        10.0,  // short_thresh
        10.0,  // long_thresh
    );
    
    // Run backtest
    let stats = backtest_signals(&result, 10000.0, 0.1);
    
    // Verify basic properties
    assert_eq!(stats.initial_budget, 10000.0);
    assert!(stats.final_budget > 0.0, "Final budget should be positive");
    assert_eq!(stats.budget_history.len(), result.prices.len());
    assert_eq!(stats.position_history.len(), result.prices.len());
    
    // Verify metrics are calculated
    assert!(stats.roi_percent.is_finite());
    assert!(stats.sharpe_ratio.is_finite());
    assert!(stats.max_drawdown >= 0.0);
    assert!(stats.win_rate >= 0.0 && stats.win_rate <= 100.0);
    
    println!("✓ Backtest integration test passed");
    println!("  Final budget: ${:.2}", stats.final_budget);
    println!("  ROI: {:.2}%", stats.roi_percent);
    println!("  Trades: {}", stats.num_trades);
    println!("  Win rate: {:.2}%", stats.win_rate);
}

#[test]
fn test_backtest_with_log_diff_generator() {
    // Create simple test data (log prices)
    let prices: Vec<f64> = (0..100)
        .map(|i| (100.0 + (i as f64 * 0.1).sin() * 5.0).ln())
        .collect();
    
    // Generate signals using the enhanced generator
    let result = generate_signals(
        "log_diff",
        &prices,
        20,    // long_lookback
        50.0,  // short_pct
        10.0,  // short_thresh
        10.0,  // long_thresh
    );
    
    // Run backtest
    let stats = backtest_signals(&result, 10000.0, 0.1);
    
    // Verify basic properties
    assert_eq!(stats.initial_budget, 10000.0);
    assert!(stats.final_budget > 0.0, "Final budget should be positive");
    
    println!("✓ Backtest with log_diff generator test passed");
    println!("  Final budget: ${:.2}", stats.final_budget);
    println!("  ROI: {:.2}%", stats.roi_percent);
}

#[test]
fn test_trade_logging() {
    // Create simple test data with clear trend
    let mut prices = Vec::new();
    for i in 0..50 {
        prices.push((100.0 + i as f64).ln());
    }
    for i in 0..50 {
        prices.push((150.0 - i as f64).ln());
    }
    
    // Generate signals
    let result = generate_signals(
        "original",
        &prices,
        10,    // long_lookback
        50.0,  // short_pct
        5.0,   // short_thresh
        5.0,   // long_thresh
    );
    
    // Run backtest
    let stats = backtest_signals(&result, 10000.0, 0.1);
    
    // Verify trade logs exist
    assert!(!stats.trades.is_empty(), "Should have some trades");
    
    // Verify trade log structure
    for trade in &stats.trades {
        assert!(trade.entry_index < trade.exit_index, "Entry should be before exit");
        assert!(trade.entry_price > 0.0, "Entry price should be positive");
        assert!(trade.exit_price > 0.0, "Exit price should be positive");
        assert!(trade.trade_type == "LONG" || trade.trade_type == "SHORT");
        assert!(trade.return_pct.is_finite());
    }
    
    println!("✓ Trade logging test passed");
    println!("  Total trades: {}", stats.trades.len());
    println!("  First trade: {} from {:.2} to {:.2}", 
             stats.trades[0].trade_type,
             stats.trades[0].entry_price,
             stats.trades[0].exit_price);
}
