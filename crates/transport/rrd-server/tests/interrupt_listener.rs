//! The interrupt listener catches an interrupt from the moment it exists.

#![cfg(unix)]

use rrd_server::Interrupt;
use std::time::Duration;

/// A server creates the listener before it announces itself. This sends the
/// process its own interrupt before the listener is ever awaited. Were the
/// listener registered only when first awaited, the signal's default action
/// would end this test process.
#[tokio::test]
async fn an_interrupt_sent_before_the_listener_is_awaited_is_caught() {
    let mut interrupt = Interrupt::listen().unwrap();

    let sent = std::process::Command::new("kill")
        .args(["-INT", &std::process::id().to_string()])
        .status()
        .expect("send this process an interrupt");
    assert!(sent.success());
    // Time for the signal to be delivered while nothing awaits the listener.
    tokio::time::sleep(Duration::from_millis(200)).await;

    tokio::time::timeout(Duration::from_secs(10), interrupt.received())
        .await
        .expect("the interrupt sent before the first await was not received");
}
