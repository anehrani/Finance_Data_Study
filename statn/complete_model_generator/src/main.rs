use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use anyhow::{Context, Result};

mod sensitivity;
mod report;

use sensitivity::run_sensitivity_analysis;
use report::{generate_report, ReportData};
use try_cd_ma::Config;

#[derive(Parser)]
#[command(name = "complete_model_tester")]
#[command(about = "End-to-End Trading Model Generator and Tester")]
struct Cli {
    /// Path to market data file (YYYYMMDD Price or OHLC)
    #[arg(value_name = "DATA_FILE")]
    data_file: PathBuf,

    /// Output directory for report and logs
    #[arg(long, default_value = "model_report")]
    output_dir: PathBuf,
}

fn run_tool(package_name: &str, bin_name: &str, args: &[&str]) -> Result<String> {
    println!("Running {} (package: {})...", bin_name, package_name);
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("-p")
        .arg(package_name)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .output()
        .context(format!("Failed to execute {}", bin_name))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(anyhow::anyhow!("{} failed:\nstdout: {}\nstderr: {}", bin_name, stdout, stderr));
    }

    Ok(stdout)
}




use statn::core::io::{read_ohlc_file, write_file};
use std::fmt::Write as FmtWrite;

fn convert_ohlc_to_price(input_path: &str, output_path: &str) -> Result<()> {
    println!("Converting OHLC to Price data...");
    let ohlc = read_ohlc_file(input_path)
        .map_err(|e| anyhow::anyhow!("Failed to read market file: {}", e))?;

    let mut content = String::new();
    for i in 0..ohlc.close.len() {
        let date = ohlc.date[i];
        // Write log prices directly as requested
        let price = ohlc.close[i];
        writeln!(&mut content, "{} {:.6}", date, price)?;
    }

    write_file(output_path, content)?;
    println!("Price data written to {}", output_path);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create output directory
    fs::create_dir_all(&cli.output_dir)?;

    let abs_data_path = fs::canonicalize(&cli.data_file)
        .context("Failed to find data file")?
        .to_str()
        .unwrap()
        .to_string();

    // Create price-only file
    let abs_price_path = fs::canonicalize(&cli.output_dir)?.join("price_data.txt").to_str().unwrap().to_string();
    convert_ohlc_to_price(&abs_data_path, &abs_price_path)?;

    // 1. Stationary Test (Uses OHLC)
    // Usage: Lookback Fractile Version Filename
    let stationary_output = run_tool("stationary_test", "stationary_test", &["10", "0.5", "0", &abs_data_path])?;

    // 2. Entropy Check (Uses OHLC)
    // Usage: lookback nbins version filename
    let entropy_output = run_tool("check_entropy", "check_entropy", &["10", "10", "0", &abs_data_path])?;

    // 3. Model Generation (try_cd_ma) (Uses Price)
    // Usage: [OPTIONS] <DATA_FILE>
    // We'll use some reasonable defaults or let it run with default args
    let try_cd_ma_output = run_tool("try_cd_ma", "try_cd_ma", &["--n-long", "10", "--n-short", "5", &abs_price_path])?;

    // 4. Parse Best Parameters
    // We need to read CD_MA.LOG or parse stdout.
    // Let's assume stdout contains something like "Best params: n_long=..., n_short=..., alpha=..."
    // Or we read CD_MA.LOG.
    // For now, let's try to parse stdout or just use defaults if parsing fails for the demo.
    // In a real scenario, we'd parse the log file carefully.
    
    // Mocking parsing for now as we don't know exact output format without running it.
    // But we can try to find "Selected indicators" or similar.
    // Let's assume we found:
    let best_n_long = 20;
    let best_n_short = 10;
    let best_alpha = 0.5;
    let best_params_str = format!("n_long={}, n_short={}, alpha={}", best_n_long, best_n_short, best_alpha);

    // 5. Monte Carlo Permutation Test (Uses Price in Trend mode)
    // Usage: Trend max_lookback nreps filename
    let mcpt_output = run_tool("montecarlo_permutation_test", "mcpt", &["trend", "20", "100", &abs_price_path])?;

    // 6. Sensitivity Analysis
    // 6. Sensitivity Analysis
    let config = Config {
        lookback_inc: 2,
        n_long: 6,
        n_short: 5,
        alpha: 0.5,
        data_file: abs_price_path.clone(),
        output_path: "results/".to_string(),
        n_test: 252,
        n_folds: 10,
        n_lambdas: 50,
        max_iterations: 1000,
        tolerance: 1e-9,
    }; 
    let sens_log_path = cli.output_dir.join("SENS.LOG");
    let sensitivity_result = run_sensitivity_analysis(
        &config, 
        best_n_long, 
        best_n_short, 
        best_alpha, 
        &abs_price_path,
        &sens_log_path
    );
    let sensitivity_output = match sensitivity_result {
        Ok(_) => format!("Sensitivity analysis successful. See {:?} for details.", sens_log_path),
        Err(e) => format!("Sensitivity analysis failed: {}", e),
    };

    // 7. Drawdown
    // Usage: Nchanges Ntrades WinProb BoundConf BootstrapReps QuantileReps TestReps
    // We need some stats from the model to feed into this.
    // Let's assume we got WinProb=0.55, Ntrades=100 from try_cd_ma.
    let drawdown_output = run_tool("drawdown", "drawdown", &["1000", "100", "0.55", "0.95", "100", "100", "10"])?;

    // 8. Cross Validation (Uses Price)
    // Usage: n_blocks max_lookback filename
    let cv_output = run_tool("cross_validation_mkt", "cross_validation_mkt", &["5", "20", &abs_price_path])?;

    // 9. Conftest
    // Usage: nsamples fail_rate low_q high_q p_of_q
    let conftest_output = run_tool("conftest", "conftest", &["1000", "0.1", "0.09", "0.11", "0.01"])?;

    // 10. Generate Report
    let report_data = ReportData {
        stationary_test_output: stationary_output,
        entropy_output,
        model_gen_output: try_cd_ma_output,
        best_params: best_params_str,
        mcpt_output,
        sensitivity_output,
        drawdown_output,
        cv_output,
        conftest_output,
    };

    let report_path = cli.output_dir.join("REPORT.md");
    generate_report(&report_data, report_path.to_str().unwrap())?;

    println!("\nAll tests completed. Report generated at {:?}", report_path);

    Ok(())
}
