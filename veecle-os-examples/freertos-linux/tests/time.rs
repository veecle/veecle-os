use assert_cmd::Command;
use predicates::boolean::PredicateBooleanExt;

#[cfg(not(miri))]
#[test]
fn test_timers() {
    Command::cargo_bin("time")
        .unwrap()
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        // Times out.
        .failure()
        .stdout(predicates::str::contains("last tick was at"))
        .stdout(predicates::str::contains("since last tick"))
        .stdout(predicates::str::contains("previous and latest tick differ in more than").not());
}
