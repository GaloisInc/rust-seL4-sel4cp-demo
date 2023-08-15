#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{memory_region_symbol, protection_domain, Channel, Handler};
use sel4cp::debug_print;

const GEM3_IRQ: Channel = Channel::new(45);
const GEM3_WAKEUP: Channel = Channel::new(46);

#[protection_domain]
fn init() -> GemHandler {
    let x= unsafe { memory_region_symbol!(gem3_register_block: *mut u64).as_ptr() };
    GemHandler { _ptr: x }
}


struct GemHandler {
    _ptr: *mut u64,
}

impl Handler for GemHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
