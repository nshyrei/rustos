pub trait FromUnsafe<T> {
    unsafe fn from_unsafe(from: T) -> Self;
}

pub trait FromAddressToStaticRef {
    unsafe fn from_unsafe(address: usize) -> &'static Self;
}