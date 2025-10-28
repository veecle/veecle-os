//! Tests for SendPolicy behavior.
//!
//! These tests verify the basic properties of SendPolicy and its interaction
//! with mpsc channels, without testing the full Output actor.

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

/// test that try_send returns an error when channel is full (drop policy)
#[tokio::test]
#[cfg_attr(coverage_nightly, coverage(off))]
async fn test_drop_policy_behavior() {
    let (tx, mut rx) = mpsc::channel::<Message<'static>>(2);

    for i in 0..2 {
        tx.send(Message::Storable(
            veecle_ipc_protocol::EncodedStorable::new(&TestData { value: i }).unwrap(),
        ))
        .await
        .unwrap();
    }

    // verify channel is full - try_send should return Err
    let result = tx.try_send(Message::Storable(
        veecle_ipc_protocol::EncodedStorable::new(&TestData { value: 100 }).unwrap(),
    ));
    assert!(result.is_err(), "try_send should fail when channel is full");

    // verify original messages are intact
    for i in 0..2 {
        let msg = rx.recv().await.unwrap();
        if let Message::Storable(data) = msg {
            let parsed: TestData = serde_json::from_str(&data.value).unwrap();
            assert_eq!(parsed.value, i);
        }
    }
}

/// test that panic is the default send policy
#[test]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_default_send_policy_is_panic() {
    assert_eq!(SendPolicy::default(), SendPolicy::Panic);
}
