use core::ops::Deref;

use tock_registers::interfaces::{Readable, Writeable, ReadWriteable};
use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

use sel4cp::debug_print;

register_structs! {
    #[allow(non_snake_case)]
    pub TtcRegisterBlock {
        (0x000 => ClockControl: ReadWrite<u32, ClockControl::Register>),
        (0x004 => _reserved0),
        (0x00C => CounterControl: ReadWrite<u32, CounterControl::Register>),
        (0x010 => _reserved1),
        (0x018 => CounterValue: ReadOnly<u32>),
        (0x01C => _reserved3),
        (0x024 => IntervalCounter: ReadWrite<u32>),
        (0x028 => _reserved4),
        (0x030 => Match1Counter1: ReadWrite<u32>),
        (0x034 => _reserved5),
        (0x03C => Match2Counter1: ReadWrite<u32>),
        (0x040 => _reserved6),
        (0x048 => Match3Counter1: ReadWrite<u32>),
        (0x04C => _reserved7),
        (0x054 => InterruptRegister: ReadWrite<u32, Interrupt::Register>), // should be ready only?
        (0x058 => _reserved8),
        (0x060 => InterruptEnable: ReadWrite<u32, Interrupt::Register>),
        (0x064 => _reserved9),
        (0x06C => EventControl: ReadWrite<u32, EventControl::Register>),
        (0x070 => _reserved10),
        (0x078 => Event: ReadOnly<u32>),
        (0x07C => @END),
    }
}

// see https://www.xilinx.com/htmldocs/registers/ug1087/ug1087-zynq-ultrascale-registers.html
register_bitfields! {
    u32,
    ClockControl [
        PS_EN OFFSET(0) NUMBITS(1) [],
        PS_V OFFSET(1)  NUMBITS(4) [],
        C_SRC OFFSET(5) NUMBITS(1) [],
        EX_E OFFSET(6) NUMBITS(1) [],
    ],
    CounterControl [
        DIS OFFSET(0) NUMBITS(1) [],
        INT OFFSET(1) NUMBITS(1) [],
        DEC OFFSET(2) NUMBITS(1) [],
        MATCH OFFSET(3) NUMBITS(1) [],
        RST OFFSET(4) NUMBITS(1) [],
        WAVE_EN OFFSET(5) NUMBITS(1) [],
        WAVE_POL OFFSET(6) NUMBITS(1) [],
    ],
    Interrupt [
        IV OFFSET(0) NUMBITS(1) [],
        M1 OFFSET(1) NUMBITS(1) [],
        M2 OFFSET(2) NUMBITS(1) [],
        M3 OFFSET(3) NUMBITS(1) [],
        OV OFFSET(4) NUMBITS(1) [],
        EV OFFSET(5) NUMBITS(1) [],
    ],
    EventControl [
        E_EN OFFSET(0) NUMBITS(1) [],
        E_LO OFFSET(1) NUMBITS(1) [],
        E_OV OFFSET(2) NUMBITS(1) [],
        E_TM OFFSET(3) NUMBITS(1) [],
    ]
}

pub struct TtcDevice {
    ptr: *const TtcRegisterBlock,
    cnt_ms: i64, // uptime in ms
}

impl TtcDevice {
    pub unsafe fn new(ptr: *const TtcRegisterBlock) -> Self {
        Self {
            ptr: ptr,
            cnt_ms: 0,
        }
    }

    fn ptr(&self) -> *const TtcRegisterBlock {
        self.ptr
    }

    pub fn init(&self) {
        // is the timer started?
        if self.CounterControl.matches_all(CounterControl::DIS::CLEAR) {
            debug_print!("Device is already started!\n");
        }
        // stop timer
        self.CounterControl.modify(CounterControl::DIS::SET);

        // initialize the device
        // Write reset value to the counter control register
        self.CounterControl.set(0x21);
        // Reset clock control
        self.ClockControl.set(0x00);
        // Reset interval count value
        self.IntervalCounter.set(0x00);
        // Reset match values
        self.Match1Counter1.set(0x00);
        self.Match2Counter1.set(0x00);
        self.Match3Counter1.set(0x00);
        // Reset IER
        self.InterruptEnable.set(0x00);
        // Reset ISR
        self.InterruptRegister.set(0x00); // according to the example drivers
        // Reset counter
        self.CounterControl.modify(CounterControl::RST::SET);

        // set options
        // set interval mode
        self.CounterControl.modify(CounterControl::INT::SET);
        // enable interval interrupt
        self.InterruptEnable.modify(Interrupt::IV::SET);
        // set interval value
        //self.IntervalCounter.set(100000000); // 10MHz is the clock frequency, should tick once a second
        self.IntervalCounter.set(100000); // 1ms clock resolution
        //self.IntervalCounter.set(20000); // 5kHz clock resolution
        //self.IntervalCounter.set(100); // 1us clock is probably too fast (the system clogs up)

        // start timer
        self.CounterControl.modify(CounterControl::DIS::CLEAR);
    }

    pub fn handle_irq(&mut self) {
        // clear interrupts
        self.InterruptRegister.get();
        self.cnt_ms = self.cnt_ms + 1;
    }

    pub fn uptime_ms(&self) -> i64 {
        self.cnt_ms
    }

}

impl Deref for TtcDevice {
    type Target = TtcRegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}