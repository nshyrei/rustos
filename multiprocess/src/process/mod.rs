use crate::Process;
use crate::Message;
use crate::ProcessBox;
use crate::executor::Executor;
use crate::executor::ExecutorRef;

use alloc::rc::Rc;
use alloc::boxed::Box;
use core::cell;
use core::ops::Deref;

pub type ProcessSystemRef = Rc<cell::RefCell<ProcessSystem>>;

pub struct ProcessRef {

    id : u64,

    executor : ProcessSystemRef
}

impl ProcessRef {

    pub fn post_message(&mut self, message : Message) {
        self.executor.borrow_mut().post_message(message)
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

struct RootProcess {
    executor : ExecutorRef,
}

impl Process for RootProcess {
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

pub struct ProcessSystem {
    executor : ExecutorRef,

    root_id : u64
}

impl ProcessSystem {

    pub fn new(mut executor : ExecutorRef) -> ProcessSystem {

        let root_process = Box::new(RootProcess { executor : Rc::clone(&executor) });

        let root_id = executor.borrow_mut().create_process(root_process);

        ProcessSystem {
            executor: Rc::clone(&executor),
            root_id
        }
    }

    fn root_id(&self) -> u64 {
        self.root_id
    }

    fn executor(&self) -> &ExecutorRef {
        &self.executor
    }

    pub fn post_message(&mut self, message : Message) {
        self.executor.borrow_mut().post_message(self.root_id, message)
    }

    pub fn fork(process_system : ProcessSystemRef, process : ProcessBox) -> ProcessRef {

        let proc_sys = process_system.borrow();
        let root_id = proc_sys.root_id();
        let executor = proc_sys.executor();

        let id = executor.borrow_mut().fork(root_id, process);

        ProcessRef {
            id,
            executor : Rc::clone(&process_system)
        }
    }
}
