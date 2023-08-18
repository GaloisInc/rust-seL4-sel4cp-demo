#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, memory_region_symbol, Channel};
use heapless::Vec;

use point_to_point_phy::PointToPointPhy;
use eth_driver_interface as interface;

const CLIENT: Channel = Channel::new(2);
const REMOTE: Channel = Channel::new(4);

#[protection_domain]
fn init() -> interface::EthHandler<PointToPointPhy> {
    unsafe {
        interface::EthHandler::new(
            CLIENT,
            REMOTE,
            PointToPointPhy::new(
                REMOTE,
                memory_region_symbol!(from_remote: *mut Vec<u8, {interface::MTU}>),
                memory_region_symbol!(to_remote: *mut Vec<u8, {interface::MTU}>),
            ),
            memory_region_symbol!(tx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_buf_region_start: *mut [interface::Buf], n = interface::TX_BUF_SIZE),
            memory_region_symbol!(rx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_buf_region_start: *mut [interface::Buf], n = interface::RX_BUF_SIZE),
        )
    }
}
