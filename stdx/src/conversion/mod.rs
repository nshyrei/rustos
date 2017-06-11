pub trait FromUnsafe<T> {
    unsafe fn from_unsafe(from: T) -> Self;
}