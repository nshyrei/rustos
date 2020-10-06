use core::marker;
use ::x86_64::port::Port;

const TEST_PASSED : u8 = 0x55;
const TEST_FAILED : u8 = 0xFC;

pub struct PS2CommandPort<T> {
    p : marker::PhantomData<T>
}

impl Port<u8> for PS2CommandPort<u8> {
    fn number() -> u16 { 0x64 }
}

impl PS2CommandPort<u8> {

    unsafe fn self_test() {
        Self::write_to_port(0xAA);
    }

    unsafe fn perform_self_test() -> bool {
        Self::self_test();
        Self::poll_for_status_ready();

        let test_result = PS2IOPort::read_port();

        test_result == TEST_PASSED
    }

    unsafe fn poll_for_status_ready() {
        let mut s = 0;
        while s & 0b00000001 != 1 {
            s = PS2CommandPort::read_port();
        }
    }
}

pub struct PS2IOPort<T> {
    p : marker::PhantomData<T>
}

impl Port<u8> for PS2IOPort<u8> {
    fn number() -> u16 { 0x60 }
}

pub unsafe fn initialize() -> bool {
    // for real hardware you need to perform more complex initialization
    // with interrupts disabled

    // clear output buffer
    PS2IOPort::read_port();

    PS2CommandPort::perform_self_test()
}