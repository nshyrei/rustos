use core::sync::atomic;
use core::ops;
use hardware::x86_64::interrupts;

pub struct Mutex<T> {

    state : atomic::AtomicBool,

    value : T
}

impl<T> Mutex<T> {

    pub fn new(value : T) -> Self {
        Mutex {
            state : atomic::AtomicBool::new(false),
            value
        }
    }

    pub fn try_acquire(&mut self) -> Option<&mut T> {
        if self.state.compare_and_swap(false, true, atomic::Ordering::Relaxed) == false {
            Some(&mut self.value)
        }
        else {
            None
        }
    }

    pub fn release(&mut self) {
        self.state.compare_and_swap(true, false, atomic::Ordering::Relaxed);
    }

    pub fn try_action<A>(&mut self, action : A) where A : FnOnce(&mut T) {
        if let Some(acquire_result) = self.try_acquire() {
            action(acquire_result);

            self.release();
        }
    }

    pub fn try_action_no_interrupts<A>(&mut self, action : A) where A : FnOnce(&mut T) {
        interrupts::disable_interrupts();

        self.try_action(action);

        interrupts::enable_interrupts();
    }

    pub fn try_action_spinlock<A>(&mut self, action : A) where A : FnOnce(&mut T) {

        let mut acquire_result = self.try_acquire();

        while acquire_result.is_none() {
            acquire_result = self.try_acquire();
        }

        action(acquire_result.unwrap());

        self.release();
    }

    pub fn try_function_spinlock<A, B>(&mut self, action : A) -> B where A : (FnOnce(&mut T) -> B) {

        let mut acquire_result = self.try_acquire();

        while acquire_result.is_none() {
            acquire_result = self.try_acquire();
        }

        let result = action(acquire_result.unwrap());

        self.release();

        result
    }
}

impl<T> ops::Deref for Mutex<T>{
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> ops::DerefMut for Mutex<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}