use std::process::Command;

fn run(expression: &str, x: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bare-jit-rs"))
        .args([expression, x])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn cli_matches_original_examples() {
    assert_eq!(run("2 + 3 * 4", "0"), "14");
    assert_eq!(run("(2 + 3) * 4", "0"), "20");
    assert_eq!(run("x * x + 2 * x + 1", "6"), "49");
}
