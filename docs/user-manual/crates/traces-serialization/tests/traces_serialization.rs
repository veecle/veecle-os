use assert_cmd::Command;

#[test]
fn traces_serialization_runs() {
    Command::cargo_bin("traces-serialization")
        .unwrap()
        .assert()
        // TODO(DEV-532): check value logged via debug format.
        .stdout(predicates::str::contains(
            r#"{"key":"type_name","value":{"String":"traces_serialization::Pong"}}"#,
        ))
        .success();
}
