static RAW_OUTPUT: &str = r"[sender] Sending 0
[sender] Sending 1
[receiver] Waiting for value
[receiver] Received: 0
[receiver] Waiting for value
[sender] Sending 2
[receiver] Received: 1
[receiver] Waiting for value
[sender] Sending 3
[receiver] Received: 2
[receiver] Waiting for value
[sender] Sending 4
[receiver] Received: 3
[receiver] Waiting for value
[sender] Sending 5
[receiver] Received: 4
[receiver] Waiting for value
[sender] Sending 6
[receiver] Received: 5
[receiver] Exiting because value is 5
";

#[test]
fn is_valid_output() {
    assert_cmd::cargo::cargo_bin_cmd!("getting-started")
        .assert()
        .stdout(RAW_OUTPUT)
        .success();
}
