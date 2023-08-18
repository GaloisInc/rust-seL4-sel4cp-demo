#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{memory_region_symbol, protection_domain, Channel, Handler};
use sel4cp::message::{MessageInfo, NoMessageValue, NoMessageLabel};
use sel4cp::debug_print;

#[link(name = "gem")]
extern "C" {
    pub fn gem_init() -> bool;
    //pub fn uartps_handle_irq();
    //pub fn uartps_rx(byte: *mut u8) -> bool;
}

const GEM3_IRQ: Channel = Channel::new(45);
const GEM3_WAKEUP: Channel = Channel::new(46);

#[protection_domain]
fn init() -> GemHandler {
    let x= unsafe { memory_region_symbol!(gem3_register_block: *mut u64).as_ptr() };
    debug_print!("[GEM] Init called!\n");
    unsafe {
        if !gem_init() {
            debug_print!("[GEM] init error.\n");
        }
    }
    GemHandler { _ptr: x }
}


struct GemHandler {
    _ptr: *mut u64,
}

impl Handler for GemHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            GEM3_IRQ => {
                GEM3_IRQ.irq_ack().unwrap();
                debug_print!("[GEM] Got irq!\n");
            }
            GEM3_WAKEUP => {
                GEM3_WAKEUP.irq_ack().unwrap();
                debug_print!("[GEM] Got a wakeup irq!\n");
            }
            _ => {
                debug_print!("[GEM] Unexpected notification from channel: {:?}\n",channel);
            }
        }
        Ok(())
    }

    fn protected(
        &mut self,
        _channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        // TODO: calls to send/receive data
        Ok(MessageInfo::send(NoMessageLabel, NoMessageValue))
    }
}
