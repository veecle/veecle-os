#[test]
fn traces_serialization_runs() {
    assert_cmd::cargo::cargo_bin_cmd!("traces-serialization")
        .assert()
        .stdout(predicates::str::contains(
            r#"{"key":"value","value":{"String":"5"}}"#,
        ))
        .success();
}
