use statn::core::io::write_file;
use std::fmt::Write as FmtWrite;

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
    let mut content = String::new();

    writeln!(&mut content, "# Complete Trading Model Report").unwrap();
    writeln!(&mut content, "\n## 1. Data Analysis").unwrap();
    writeln!(&mut content, "### Stationary Test\n```\n{}\n```", data.stationary_test_output).unwrap();
    writeln!(&mut content, "### Entropy Analysis\n```\n{}\n```", data.entropy_output).unwrap();

    writeln!(&mut content, "\n## 2. Model Generation (try_cd_ma)").unwrap();
    writeln!(&mut content, "### Output Summary\n```\n{}\n```", data.model_gen_output).unwrap();
    writeln!(&mut content, "### Best Parameters\n{}", data.best_params).unwrap();

    writeln!(&mut content, "\n## 3. Model Verification").unwrap();
    writeln!(&mut content, "### Monte Carlo Permutation Test\n```\n{}\n```", data.mcpt_output).unwrap();
    writeln!(&mut content, "### Sensitivity Analysis\n```\n{}\n```", data.sensitivity_output).unwrap();
    writeln!(&mut content, "### Drawdown Analysis\n```\n{}\n```", data.drawdown_output).unwrap();
    writeln!(&mut content, "### Cross Validation\n```\n{}\n```", data.cv_output).unwrap();
    writeln!(&mut content, "### Confidence Test (Conftest)\n```\n{}\n```", data.conftest_output).unwrap();

    write_file(path, content)
}
