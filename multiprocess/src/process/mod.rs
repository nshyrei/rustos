use crate::Process;
use crate::Message;
use crate::ProcessBox;
use crate::executor::Executor;
use crate::executor::ExecutorRef;

use alloc::rc::Rc;
use alloc::boxed::Box;
use core::cell;
use core::ops::Deref;
use core::clone::Clone;

pub struct ProcessRef {

    id : u64,

    executor: ExecutorRef
}

impl ProcessRef {

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn fork(&mut self, process : ProcessBox) -> ProcessRef {
unsafe {
    let id = self.executor.get().as_mut().unwrap()/*.borrow_mut()*/.fork(self.id, process);

    ProcessRef {
        id,
        executor: Rc::clone(&self.executor)
    }
}
    }

    pub fn post_message(&mut self, message : Message) {
        unsafe { self.executor.get().as_mut().unwrap().post_message(self.id, message) }
    }
}

impl Clone for ProcessRef {
    fn clone(&self) -> Self {
        ProcessRef {
            id : self.id,
            executor: Rc::clone(&self.executor)
        }
    }
}

pub struct RemoveProcess {
    id : u64
}

pub struct StartProcess {}

pub struct CreateProcess {

    pub parent : u64,

    pub process_message : ProcessBox
}

pub struct RootProcess {
    executor : ExecutorRef,
}

impl RootProcess {
    pub fn new(executor : ExecutorRef) -> ProcessRef {
        unsafe {
            let root_process = RootProcess { executor: Rc::clone(&executor) };
            let root_process_box = Box::new(root_process);

            let id = executor.get().as_mut().unwrap().create_process(root_process_box);

            ProcessRef {
                id,
                executor: Rc::clone(&executor)
            }
        }
    }
}

impl Process for RootProcess {

    fn process_message(&mut self, message: Message) -> () {
        unsafe {
            if message.is::<CreateProcess>() {
                let msg = message.downcast::<CreateProcess>().unwrap();

                self.executor.get().as_mut().unwrap().create_process(msg.process_message);
            } else if message.is::<RemoveProcess>() {
                let msg = message.downcast::<RemoveProcess>().unwrap();

                self.executor.get().as_mut().unwrap().remove_process_with_children(msg.id);
            }
        }
    }
}
