use assert_cmd::Command;

#[test]
fn test_alloc() {
    Command::cargo_bin("alloc")
        .unwrap()
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        // Times out.
        .failure()
        .stdout(predicates::str::contains(
            r#""Boxes","attributes":[{"key":"boxes","value":{"String":"[Some(0), Some(1), Some(2), Some(3), Some(4)]"}}]"#,
        ));
}
