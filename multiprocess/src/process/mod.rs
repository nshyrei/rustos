use crate::executor::Executor;

use alloc::rc::Rc;
use core::cell;
use core::ptr;
use core::ops::Deref;
use core::ops::DerefMut;
use core::clone::Clone;
use core::any::Any;
use core::marker::Copy;
use core::default::Default;
use alloc::boxed::Box;

pub type Message = Box<Any>;

pub type ProcessBox = Box<Process>;

pub struct Terminate { }

pub struct Crashed {
    reason : &'static str
}

pub trait Process {

    fn set_id(&mut self, id : u64) -> () {}

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
    executor : ptr::NonNull<Executor>,
}

impl RootProcess {
    pub fn new(executor : &mut Executor) -> Self {
        unsafe {
            RootProcess {
                executor: ptr::NonNull::new_unchecked(executor)
            }
        }
    }
}

impl Process for RootProcess {

    fn description(&self) -> &'static str {
        "Root process of the system."
    }

    fn process_message(&mut self, message: Message) -> () {

    }
}

#[derive(Copy, Clone)]
pub struct KeyboardPress {
    pub code : u8
}

#[derive(Copy, Clone)]
pub struct SubscribeMe {
    pub id : u64
}

pub struct KickStart {}

use alloc::vec::Vec;

pub struct HardwareListener {

    pub id : u64,

    executor : ptr::NonNull<Executor>,

    subscribers : Vec<u64>,

    event_queue : Vec<KeyboardPress>
}

impl HardwareListener {
    pub fn new(executor : &mut Executor) -> Self {
        let event_queue = Vec::<KeyboardPress>::new();
        let subscribers = Vec::<u64>::new();

        unsafe {
            HardwareListener {
                id : 0,
                executor: ptr::NonNull::new_unchecked(executor),
                subscribers,
                event_queue
            }
        }
    }
}

impl Process for HardwareListener {

    fn set_id(&mut self, id : u64) -> () {
        self.id = id;
    }

    fn description(&self) -> &'static str {
        "Process that listens to hardware events and propagates them to subscribers."
    }

    fn process_message(&mut self, message: Message) -> () {
        if message.is::<KeyboardPress>() {
            let unwr = message.downcast::<KeyboardPress>().unwrap();

            self.event_queue.push(*unwr);
        }
        else if message.is::<SubscribeMe>() {
            let sub = message.downcast::<SubscribeMe>().unwrap();

            self.subscribers.push(sub.id);
        }

        if self.subscribers.len() != 0 && self.event_queue.len() != 0 {
            let msg = self.event_queue
                .pop()
                .unwrap();

            for e in &self.subscribers {

                let cpy = Box::new(msg.clone());

               unsafe { self.executor.as_mut().post_message(*e, cpy); }
            }
        }
    }
}
