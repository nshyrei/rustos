use core::sync::atomic;
use hardware::x86_64::interrupts;

pub struct Mutex<T> {

    state : atomic::AtomicBool,

    value : T
}

impl<T> Mutex<T> {

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

}