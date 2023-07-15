#![no_std]

use zerocopy::{AsBytes, FromBytes};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum TimerTag {
    Sleep,
    Uptime,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct SleepRequest {
    pub ms: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
// This is basically an unpacked Duration struct, since we cannot
// automatically derive AsBytes, FromBytes for `Duration` with
// private fields
pub struct UptimeValue {
    pub seconds: i64,
    pub nanoseconds: i64, // to avoid padding
}
