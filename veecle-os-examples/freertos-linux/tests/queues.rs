use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_queues() {
    Command::cargo_bin("queues")
        .unwrap()
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        // Times out.
        .failure()
        .stdout(
            predicates::str::contains("[Legacy task] Got data back").and(
                predicates::str::contains("[Veecle OS task] Received some async data"),
            ),
        );
}
