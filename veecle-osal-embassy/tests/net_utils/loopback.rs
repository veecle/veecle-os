use embassy_net_driver::{Capabilities, HardwareAddress, LinkState};
use std::collections::VecDeque;
use std::task::Context;
use std::vec;
use std::vec::Vec;

// Taken from https://docs.rs/smoltcp/latest/src/smoltcp/phy/loopback.rs.html
// and adapted to the `embassy_net_driver::Driver` API.

/// A loopback device.
#[derive(Debug)]
pub struct Loopback {
    pub(crate) queue: VecDeque<Vec<u8>>,
}

#[allow(clippy::new_without_default)]
impl Loopback {
    pub fn new() -> Loopback {
        Loopback {
            queue: VecDeque::new(),
        }
    }
}

impl embassy_net_driver::Driver for Loopback {
    type RxToken<'a> = RxToken;

    type TxToken<'a> = TxToken<'a>;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.queue.pop_front().map(move |buffer| {
            let rx = RxToken { buffer };

            let tx = TxToken {
                queue: &mut self.queue,
            };

            cx.waker().wake_by_ref();

            (rx, tx)
        })
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        cx.waker().wake_by_ref();
        Some(TxToken {
            queue: &mut self.queue,
        })
    }

    fn link_state(&mut self, _cx: &mut Context) -> LinkState {
        LinkState::Up
    }

    fn capabilities(&self) -> Capabilities {
        let mut capabilities = Capabilities::default();
        capabilities.max_transmission_unit = 65535;
        capabilities.max_burst_size = None;
        capabilities
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ip
    }
}

#[derive(Debug)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl embassy_net_driver::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

#[derive(Debug)]
pub struct TxToken<'a> {
    queue: &'a mut VecDeque<Vec<u8>>,
}

impl<'a> embassy_net_driver::TxToken for TxToken<'a> {
    fn consume<R, F>(self, length: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0; length];

        let result = f(&mut buffer);

        self.queue.push_back(buffer);

        result
    }
}
