//! Tests for `SendPolicy` behavior.
//!
//! These tests verify the basic properties of `SendPolicy` and its interaction
//! with `mpsc` channels, without testing the full `Output` actor.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use veecle_ipc::SendPolicy;
use veecle_ipc_protocol::Message;
use veecle_os_runtime::Storable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Storable, Serialize, Deserialize)]
struct TestData {
    value: u32,
}

/// Test that `try_send` returns an error when channel is full (drop policy).
#[tokio::test]
#[cfg_attr(coverage_nightly, coverage(off))]
async fn test_drop_policy_behavior() {
    let (sender, mut receiver) = mpsc::channel::<Message>(2);

    for index in 0..2 {
        sender
            .send(Message::Storable(
                veecle_ipc_protocol::EncodedStorable::new(&TestData { value: index }).unwrap(),
            ))
            .await
            .unwrap();
    }

    // Verify channel is full - `try_send` should return `Err`.
    let result = sender.try_send(Message::Storable(
        veecle_ipc_protocol::EncodedStorable::new(&TestData { value: 100 }).unwrap(),
    ));
    assert!(result.is_err(), "try_send should fail when channel is full");

    // Verify original messages are intact.
    for index in 0..2 {
        let message = receiver.recv().await.unwrap();
        if let Message::Storable(data) = message {
            let parsed: TestData = serde_json::from_str(&data.value).unwrap();
            assert_eq!(parsed.value, index);
        }
    }
}

/// Test that panic is the default send policy.
#[test]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_default_send_policy_is_panic() {
    assert_eq!(SendPolicy::default(), SendPolicy::Panic);
}
