use std::process::Command;

#[test]
fn test_conftest_runs() {
    // We need to run the binary. Since this is an integration test within the package,
    // we can assume `cargo run` works, but it might be better to find the binary.
    // However, `cargo run` inside a test might be recursive/slow.
    // A better way is to use `assert_cmd` crate if available, but I don't want to add dependencies.
    // I'll just use `cargo run` as it's simple.
    
    let output = Command::new("cargo")
        .args(&["run", "-p", "conftest", "--", "100", "0.1", "0.05", "0.15", "0.01"])
        .env("CONFTEST_MAX_ITERS", "5")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nsamps=100"));
    assert!(stdout.contains("Lower bound fail above="));
}
