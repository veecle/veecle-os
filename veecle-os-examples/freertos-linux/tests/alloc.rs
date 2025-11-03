#[test]
fn test_alloc() {
    assert_cmd::cargo::cargo_bin_cmd!("alloc")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        // Times out.
        .failure()
        .stdout(predicates::str::contains(
            r#""Boxes","attributes":[{"key":"boxes","value":{"String":"[Some(0), Some(1), Some(2), Some(3), Some(4)]"}}]"#,
        ));
}
