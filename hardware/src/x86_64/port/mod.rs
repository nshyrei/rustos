pub trait PortRead {
    //#[inline(always)]
    unsafe fn read(port_number : u16) -> Self;
}

pub trait PortWrite {
    //#[inline(always)]
    unsafe fn write(port_number : u16, arg : Self);
}

pub trait PortReadWrite : PortRead + PortWrite { }

impl PortWrite for u8 {
    unsafe fn write(port_number: u16, value: u8) {
        asm!("outb $1, $0" :: "N{dx}"(port_number), "{al}"(value) :: "volatile");
    }
}

impl PortRead for u8 {
    unsafe fn read(port_number: u16) -> u8 {
        let value : u8;
        asm!("inb $1, $0" : "={al}"(value) : "N{dx}"(port_number) :: "volatile");
        value
    }
}

impl PortReadWrite for u8 {}

pub trait Port<T : PortReadWrite> {
    fn number() -> u16;

    unsafe fn read_port() -> T {
        T::read(Self::number())
    }

    unsafe fn write_to_port(value : T) {
        T::write(Self::number(), value)
    }
}
