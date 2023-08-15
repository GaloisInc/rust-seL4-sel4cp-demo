//! An example how to get current Uptime from the timer:
//! ```rust
//! let msg_info = TIMER
//!     .pp_call(MessageInfo::send(TimerRequest::Uptime, NoMessageValue));
//! match msg_info.label().try_into() {
//!    Ok(TimerRequest::Uptime) => match msg_info.recv() {
//!        Ok(UptimeValue { ms }) => {
//!            debug_print!("[ETH] Uptime is {} ms\n",ms);
//!        },
//!        Err(e) => {
//!            debug_print!("[ETH] Message receive error {:?}\n",e);
//!        },
//!    }
//!    _ => {
//!        debug_print!("[ETH] Unexpected reply\n");
//!    },
//! }
//! ```
#![no_std]

use zerocopy::{AsBytes, FromBytes};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum TimerRequest {
    Sleep,
    Uptime,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct SleepRequest {
    pub ms: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
// This is basically an unpacked Duration struct, since we cannot
// automatically derive AsBytes, FromBytes for `Duration` with
// private fields
pub struct UptimeValue {
    pub ms: i64,
}
