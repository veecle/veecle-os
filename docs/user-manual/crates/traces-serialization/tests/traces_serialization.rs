#[test]
fn traces_serialization_runs() {
    assert_cmd::cargo::cargo_bin_cmd!("traces-serialization")
        .assert()
        // TODO(DEV-532): check value logged via debug format.
        .stdout(predicates::str::contains(
            r#"{"key":"type_name","value":{"String":"traces_serialization::Pong"}}"#,
        ))
        .success();
}
