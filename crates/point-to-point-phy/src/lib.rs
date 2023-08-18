#![no_std]

use sel4cp::Channel;
use sel4cp::memory_region::{ExternallySharedRef, ExternallySharedPtr, ReadWrite};
use smoltcp::{phy, time::Instant};
use heapless::Vec;

use eth_driver_interface::MTU;

pub struct PointToPointPhy {
    channel: Channel,
    rx_buf: ExternallySharedRef<'static, Vec<u8, MTU>>,
    tx_buf: ExternallySharedRef<'static, Vec<u8, MTU>>,
}

impl PointToPointPhy {
    pub fn new(
        channel: Channel,
        rx_buf_ptr: core::ptr::NonNull<Vec<u8, MTU>>,
        tx_buf_ptr: core::ptr::NonNull<Vec<u8, MTU>>,
    ) -> Self {
        let rx_buf = unsafe { ExternallySharedRef::<'static, Vec<u8, MTU>>::new(rx_buf_ptr) };
        let tx_buf = unsafe { ExternallySharedRef::<'static, Vec<u8, MTU>>::new(tx_buf_ptr) };

        Self { channel, rx_buf, tx_buf }
    }
}

pub struct TxToken<'a> {
    channel: Channel,
    buf: ExternallySharedPtr<'a, Vec<u8, MTU>, ReadWrite>,
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(mut self, length: usize, f: F) -> R {
        let mut tmp = Vec::<u8, MTU>::default();

        tmp.extend(core::iter::repeat(0).take(length));
        let res = f(&mut tmp);

        let tx_buf_mut = unsafe { self.buf.as_raw_ptr().as_mut() };
        tx_buf_mut.clear();
        let _ = tx_buf_mut.extend_from_slice(&tmp);
        let _ = tx_buf_mut.resize(length, 0);

        // Notify the other endpoint that there's something to read
        self.channel.notify();

        res
    }
}

pub struct RxToken {
    buf: Vec<u8, MTU>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(mut self, f: F) -> R {
        f(&mut self.buf)
    }
}

impl phy::Device for PointToPointPhy {
    type TxToken<'a> = TxToken<'a>;
    type RxToken<'a> = RxToken;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx_buf = unsafe {
            self.rx_buf
                .as_ptr()
                .as_raw_ptr()
                .as_ref()
                .clone()
        };

        let tx_buf = self.tx_buf.as_mut_ptr();

        Some((
            RxToken {
                buf: rx_buf,
            },
            TxToken {
                channel: self.channel,
                buf: tx_buf
            }
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let tx_buf = self.tx_buf.as_mut_ptr();

        Some(
            TxToken {
                channel: self.channel,
                buf: tx_buf
            }
        )
    }

    fn capabilities(&self) -> phy::DeviceCapabilities {
        let mut caps = phy::DeviceCapabilities::default();
        caps.medium = phy::Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = None;
        caps.checksum.ipv4 = phy::Checksum::None;
        caps.checksum.udp = phy::Checksum::None;
        caps.checksum.tcp = phy::Checksum::None;
        caps.checksum.icmpv4 = phy::Checksum::None;

        caps
    }
}
