use assert_cmd::Command;

#[test]
fn test_basic() {
    Command::cargo_bin("basic")
        .unwrap()
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        // Times out.
        .failure()
        .stdout(predicates::str::contains("Hello from Task!"));
}
