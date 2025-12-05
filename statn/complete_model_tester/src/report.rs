use std::fs::File;
use std::io::Write;

pub struct ReportData {
    pub stationary_test_output: String,
    pub entropy_output: String,
    pub model_gen_output: String,
    pub best_params: String,
    pub mcpt_output: String,
    pub sensitivity_output: String,
    pub drawdown_output: String,
    pub cv_output: String,
    pub conftest_output: String,
}

pub fn generate_report(data: &ReportData, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "# Complete Trading Model Report")?;
    writeln!(file, "\n## 1. Data Analysis")?;
    writeln!(file, "### Stationary Test\n```\n{}\n```", data.stationary_test_output)?;
    writeln!(file, "### Entropy Analysis\n```\n{}\n```", data.entropy_output)?;

    writeln!(file, "\n## 2. Model Generation (try_cd_ma)")?;
    writeln!(file, "### Output Summary\n```\n{}\n```", data.model_gen_output)?;
    writeln!(file, "### Best Parameters\n{}", data.best_params)?;

    writeln!(file, "\n## 3. Model Verification")?;
    writeln!(file, "### Monte Carlo Permutation Test\n```\n{}\n```", data.mcpt_output)?;
    writeln!(file, "### Sensitivity Analysis\n```\n{}\n```", data.sensitivity_output)?;
    writeln!(file, "### Drawdown Analysis\n```\n{}\n```", data.drawdown_output)?;
    writeln!(file, "### Cross Validation\n```\n{}\n```", data.cv_output)?;
    writeln!(file, "### Confidence Test (Conftest)\n```\n{}\n```", data.conftest_output)?;

    Ok(())
}
