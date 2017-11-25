pub trait Monoid {
    fn zero() -> Self;

    fn append(&self, other : Self) -> Self;
}

impl Monoid for u32 {
    fn zero() -> u32 {
        0
    }

    fn append(&self, other : u32) -> u32 {
        self + other
    }
}

impl Monoid for u64 {
    fn zero() -> u64 {
        0
    }

    fn append(&self, other : u64) -> u64 {
        self + other
    }
}


impl Monoid for usize {
    fn zero() -> usize {
        0
    }

    fn append(&self, other : usize) -> usize {
        self + other
    }
}