use crate::Process;
use crate::Message;
use crate::ProcessBox;
use crate::executor::Executor;
use crate::executor::ExecutorRef;

use alloc::rc::Rc;
use alloc::boxed::Box;
use core::cell;

pub struct ProcessRef {

    id : u64,

    executor : ProcessSystemRef
}

impl ProcessRef {

    pub fn post_message(&mut self, message : Message) {
        self.executor.post_message(message)
    }
}

struct RemoveProcess {
    id : u64
}

struct StartProcess {}

struct CreateProcess {

    parent : u64,

    process_message : ProcessBox
}

pub struct ProcessSystem {
    executor : ExecutorRef
}

impl ProcessSystem {

    fn new(executor : ExecutorRef) -> ProcessSystem {
        ProcessSystem { executor }
    }

    fn reff(&self) -> ProcessSystemRef {
        ProcessSystemRef {
            id : 0,
            executor : Rc::clone(&self.executor)
        }
    }
}

impl Process for ProcessSystem {
    fn process_message(&mut self, message: Message) -> () {
        if message.is::<CreateProcess>() {
            let msg = message.downcast::<CreateProcess>().unwrap();

            self.executor.borrow_mut().create_process(msg.process_message);
        } else if message.is::<RemoveProcess>() {
            let msg = message.downcast::<RemoveProcess>().unwrap();

            self.executor.borrow_mut().remove_process_with_children(msg.id);
        }
    }
}

pub struct ProcessSystemRef {
    id : u64,

    executor : ExecutorRef
}

impl ProcessSystemRef {

    pub fn post_message(&mut self, message : Message) {
        self.executor.borrow_mut().post_message(self.id, message)
    }

    pub fn fork(&mut self, process : ProcessBox) -> ProcessRef {

        let id = self.executor.borrow_mut().fork(self.id, process);
        let executor = Rc::clone(&self.executor);

        let process_system = ProcessSystemRef {
            id : self.id,
            executor
        };

        ProcessRef {
            id,
            executor : process_system
        }
    }
}



pub struct SampleProcess {

    executor : ProcessSystemRef,

    child : ProcessRef
}

impl Process for SampleProcess {
    fn process_message(&mut self, message: Message) -> () {
        if message.is::<CreateProcess>() {
            let msg = message.downcast::<CreateProcess>().unwrap();

            let new_child = self.executor.fork(msg.process_message);
            self.child = new_child;
        }
    }
}
