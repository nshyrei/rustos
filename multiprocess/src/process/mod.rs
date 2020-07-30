use crate::executor::Executor;

use alloc::rc::Rc;
use core::cell;
use core::ptr;
use core::ops::Deref;
use core::clone::Clone;
use core::any::Any;
use core::default::Default;
use alloc::boxed::Box;

pub type Message = Box<dyn Any>;

pub type ProcessBox = Box<dyn Process>;

pub struct Terminate { }

pub trait Process {

    fn description(&self) -> &'static str;

    fn process_message(&mut self, message : Message) -> ();
}

pub struct ProcessRef {

    id : u64,

    executor: ptr::NonNull<Executor>
}

impl ProcessRef {

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn fork(&mut self, process: ProcessBox) -> ProcessRef {
        unsafe {
            let id = self.executor.as_mut().fork(self.id, process);

                ProcessRef {
                    id,
                    executor: self.executor,
                }
        }
    }

    pub fn post_message(&mut self, message : Message) {
        unsafe { self.executor.as_mut().post_message(self.id, message) }
    }
}

impl Clone for ProcessRef {
    fn clone(&self) -> Self {
        ProcessRef {
            id : self.id,
            executor: self.executor
        }
    }
}

pub struct RootProcess {

}

impl RootProcess {
    pub fn new(executor : &mut Executor) -> ProcessRef {
        unsafe {
            let root_process = RootProcess {  };
            let root_process_box = Box::new(root_process);

            let id = executor.create_process(root_process_box);

            ProcessRef {
                id,
                executor: ptr::NonNull::new_unchecked(executor)
            }
        }
    }
}

impl Process for RootProcess {

    fn description(&self) -> &'static str {
        "Root process."
    }

    fn process_message(&mut self, message: Message) -> () {
        /*unsafe {
            if message.is::<CreateProcess>() {
                let msg = message.downcast::<CreateProcess>().unwrap();

                self.executor.get().as_mut().unwrap().create_process(msg.process_message);
            } else if message.is::<RemoveProcess>() {
                let msg = message.downcast::<RemoveProcess>().unwrap();

                self.executor.get().as_mut().unwrap().remove_process_with_children(msg.id);
            }
        }*/
    }
}
