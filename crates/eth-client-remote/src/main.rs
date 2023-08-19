#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4cp::debug_print;

use smoltcp::phy::{Device, TxToken, RxToken};
use smoltcp::wire::{IpEndpoint, IpAddress, IpCidr, EthernetAddress};
use smoltcp::storage::PacketMetadata;
use smoltcp::time::Instant;
use smoltcp::socket::{udp, tcp};

#[allow(unused_imports)]
use eth_driver_interface as interface;
use smoltcp::iface;

const DRIVER: Channel = Channel::new(5);
const ETH_TEST: Channel = Channel::new(6);

const PONG: [u8; 4] = ['P' as u8, 'O' as u8, 'N' as u8, 'G' as u8];

#[protection_domain]
fn init() -> ThisHandler {
    let mut device = unsafe {
        interface::EthDevice::new(
            DRIVER,
            memory_region_symbol!(tx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_buf_region_start: *mut [interface::Buf], n = interface::TX_BUF_SIZE),
            memory_region_symbol!(rx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_buf_region_start: *mut [interface::Buf], n = interface::RX_BUF_SIZE),
        )
    };

    let netcfg = iface::Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());

    let mut netif = iface::Interface::new(netcfg, &mut device, Instant::from_millis(100));
    netif.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8))
            .unwrap(); // TODO Handle this error
        });

    // `cnt` will simulate system clock
    let cnt = 0;
    ThisHandler{
        device,
        netif,
        cnt,
    }
}


struct ThisHandler{
    device: interface::EthDevice,
    netif: iface::Interface,
    cnt: u32,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            ETH_TEST => {
                self.cnt = self.cnt + 100;
                debug_print!("Remote client got notification!\n");

                test_ethernet_pong(self);
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

fn test_ethernet_pong(h: &mut ThisHandler) {
    debug_print!("Testing ethernet pong\n");

    match h.device.receive(Instant::from_millis(h.cnt)) {
        None => debug_print!("[test_ethernet_pong] No RX token received\n"),
        Some((rx, _tx)) => {
            rx.consume(|buffer|
                debug_print!(
                    "[test_ethernet_pong] Got an RX token of length {}: {}\n",
                    buffer.len(),
                    core::str::from_utf8(buffer).unwrap(),
                )
            );
        }
    }
    match h.device.transmit(Instant::from_millis(h.cnt)) {
        None => debug_print!("[test_ethernet_pong] Didn't get a TX token\n"),
        Some(tx) => {
            debug_print!(
                "[test_ethernet_pong] Got a TX token\nSending some data: {}\n",
                core::str::from_utf8(&PONG).unwrap(),
            );
            tx.consume(4, |buffer| buffer.copy_from_slice(PONG.as_ref()))
        }
    }
}
