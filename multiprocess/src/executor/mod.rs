use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;
use core::ptr;
use core::ops;

use crate::Message;
use crate::ProcessBox;
use crate::Process;

pub type ExecutorRef = Rc<cell::UnsafeCell<Executor>>;

pub struct ExecutorHelp {
    pub value: ptr::NonNull<ExecutorRef>
}

impl ops::Deref for ExecutorHelp {
    type Target = Executor;

    fn deref(&self) -> &Executor {
        unsafe { self.value.as_ref().get().as_ref().unwrap() }
    }
}

impl ops::DerefMut for ExecutorHelp {
    fn deref_mut(&mut self) -> & mut Executor {
        unsafe { self.value.as_mut().get().as_mut().unwrap() }
    }
}

pub struct Executor {
    id_counter: u64,

    currently_executing: u64,

    execution_line: VecDeque<u64>,

    existing: BTreeMap<u64, ProcessDescriptor>,
}

impl Executor {
    pub fn new() -> Self {
        let id_counter = 0;
        let execution_line: VecDeque<u64> = VecDeque::new();
        let existing: BTreeMap<u64, ProcessDescriptor> = BTreeMap::new();

        Executor {
            id_counter,
            currently_executing: 0,
            execution_line,
            existing,
        }
    }

    pub fn post_message(&mut self, id: u64, message: Message) {
        if let Some(process) = self.existing.get_mut(&id) {
            process.mailbox.push_back(message)
        }
    }

    pub(crate) fn remove_process_with_children(&mut self, id: u64) {
        if let Some(node) = self.existing.remove(&id) {
            for child_id in node.children {
                self.remove_process_with_children(child_id);
            }
        }
    }

    pub(crate) fn remove_process(&mut self, id: u64) {
        self.existing.remove(&id);
    }

    pub fn create_process(&mut self, process_message: ProcessBox) -> u64 {
        let node = ProcessDescriptor::new(process_message);
        let id = self.id_counter;

        self.existing.insert(id, node);
        self.execution_line.push_back(id);
        self.id_counter += 1;

        id
    }

    /*pub (crate) fn fork(&mut self, parent_id : u64, process_message : ProcessBox) -> u64 {
        if self.existing.contains_key(&parent_id) {
            let child_id = self.create_process(process_message);

            let parent_node = self.existing.get_mut(&parent_id).unwrap();

            parent_node.children.push(child_id);

            child_id
        }
        else {
            0
        }
    }*/

    pub fn update_current_process(&mut self, interrupted_process_state: ProcessRegisters) {
        if let Some(existing_process) = self.existing.get_mut(&self.currently_executing) {

            if existing_process.state == ProcessState::Running {
                existing_process.registers = interrupted_process_state;
            }


        }
    }

    pub fn schedule_next(&mut self) -> Option<&mut ProcessDescriptor> {
        // Round robin algorithm: consecutively execute processes without any regard to priorities or round-trip time
        // pick one process to execute from execution line,
        // execute it and put it back into the queue

        // put process to the back the queue for next round of scheduling
        self.execution_line.push_back(self.currently_executing);

        self.execution_line.pop_front().and_then(move |head_id| {
            self.currently_executing = head_id;

            self.existing.get_mut(&head_id)
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    New,
    Running,
    Finished,
}

#[repr(C)]
pub struct ProcessDescriptor {
    process: ProcessBox,

    // 1 page frame
    stack : Box<[u8; 4096]>,

    mailbox: VecDeque<Message>,

    children: Vec<u64>,

    state: ProcessState,

    registers: ProcessRegisters,
}

#[derive(Copy, Clone, Debug)]
pub struct ProcessRegisters {
    pub instruction_pointer: u64,

    pub stack_pointer: u64,

    pub cpu_flags: u64,
}

impl ProcessDescriptor {
    fn new(process: ProcessBox) -> Self {
        let mailbox: VecDeque<Message> = VecDeque::new();
        let children: Vec<u64> = Vec::new();
        let state = ProcessState::New;
        let stack = Box::new([0 as u8; 4096]);

        use core::ops::Deref;
        let stack_pointer = (&stack as *const _ as u64 + 4096);

        let registers = ProcessRegisters {
            instruction_pointer: 0, // process function will be called directly and this value will be populated after interrupt
            stack_pointer,
            cpu_flags: 0,
        };

        ProcessDescriptor {
            process,
            stack,
            mailbox,
            children,
            state,
            registers,
        }
    }

    pub fn mailbox_mut(&mut self) -> &mut VecDeque<Message> {
        &mut self.mailbox
    }

    pub fn mailbox(&self) -> &VecDeque<Message> {
        &self.mailbox
    }

    pub fn registers(&self) -> &ProcessRegisters {
        &self.registers
    }

    pub fn state(&self) -> &ProcessState {
        &self.state
    }

    pub fn stack_address(&self) -> u64 {
        use core::ops::Deref;
        self.stack.deref() as *const _ as u64 + 4096
    }

    pub fn process(&mut self) -> &mut ProcessBox {
        &mut self.process
    }
pub fn pop_pront(&mut self) -> () {
    self.mailbox.pop_front();
    }
    pub fn switch(&mut self) -> () {
        if let Some(message) = self.mailbox.pop_front() {
            self.state = ProcessState::Running;
            self.process.process_message(message);
        }
    }

    pub unsafe fn unsafe_box(&mut self) -> *mut Process {
        use core::mem;
        use core::ptr;
        let rawNull = ptr::read_unaligned(&mut self.process);

        let raw = Box::into_raw(rawNull);

        raw
    }

    pub fn process_addr(&self) -> &ProcessBox {
        &self.process
    }
}