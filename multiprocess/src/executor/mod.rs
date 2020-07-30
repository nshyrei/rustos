use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;
use core::ptr;
use core::ops;
use core::cmp;
use core::any::Any;

use crate::process::{
    Message,
    Terminate,
    ProcessBox,
    Process
};

pub struct Executor {
    id_counter: u64,

    currently_executing: u64,

    execution_line: BinaryHeap<ProcessorTimeWithProcessKey>,

    existing: BTreeMap<u64, ProcessDescriptor>,
}

impl Executor {
    pub fn new() -> Self {
        let id_counter = 0;
        let execution_line: BinaryHeap<ProcessorTimeWithProcessKey> = BinaryHeap::new();
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

    pub fn remove_currently_executing(&mut self) {
        self.remove_process(self.currently_executing);
    }

    pub fn currently_executing_descriptor(&self) -> &ProcessDescriptor {
        self.existing.get(&self.currently_executing).unwrap()
    }

    pub fn currently_executing(&self) -> u64 {
        self.currently_executing
    }

    pub(crate) fn create_process(&mut self, process_message: ProcessBox) -> u64 {

        let mut node = ProcessDescriptor::new(process_message, self.existing.len() as u64);
        //node.create_guard();
        let id = self.id_counter;

        self.existing.insert(id, node);

        let new_entry = ProcessorTimeWithProcessKey {
            processor_time : 0,
            process_key: id
        };

        self.execution_line.push(new_entry);
        self.id_counter += 1;

        id
    }

    pub (crate) fn fork(&mut self, parent_id : u64, process_message : ProcessBox) -> u64 {
        if self.existing.contains_key(&parent_id) {
            let child_id = self.create_process(process_message);

            let parent_node = self.existing.get_mut(&parent_id).unwrap();

            parent_node.children.push(child_id);

            child_id
        }
        else {
            0
        }
    }

    pub fn save_interrupted_process_return_point(&mut self, interrupted_process_registers: ProcessRegisters) {
        if let Some(existing_process) = self.existing.get_mut(&self.currently_executing) {

            if existing_process.state == ProcessState::Running {
                existing_process.registers = interrupted_process_registers;
            }
        }
    }

    pub fn schedule_next(&mut self, time : u64) -> Option<&mut ProcessDescriptor> {
        let process = self.existing.get(&self.currently_executing).unwrap();

        let execution_period = time - process.execution_start_time;

        match process.state() {
            ProcessState::AskedToTerminate => {
                self.existing.remove(&self.currently_executing);
            },
            ProcessState::Running if execution_period < process.maximum_execution_time => {
                let processor_time = if execution_period < process.maximum_execution_time {
                    execution_period
                }
                else {
                    // add some execution time, so finished process wont get bumped back into the top of queue
                    process.maximum_execution_time / (self.existing.len() as u64)
                };

                let new_entry = ProcessorTimeWithProcessKey {
                    processor_time,
                    process_key: self.currently_executing
                };

                self.execution_line.push(new_entry)
            }
            _ => ()
        }

        self.set_queue_front_as_executing()
    }

    fn set_queue_front_as_executing(&mut self) -> Option<&mut ProcessDescriptor> {
        self.execution_line.pop().and_then(move |head| {
            let head_id = head.process_key;

            self.currently_executing = head_id;

            self.existing.get_mut(&head_id)
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProcessorTimeWithProcessKey {
    processor_time: u64,

    process_key : u64
}

impl cmp::PartialOrd for ProcessorTimeWithProcessKey {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.processor_time.partial_cmp(&other.processor_time).map(|o| o.reverse())
    }
}

impl cmp::Ord for ProcessorTimeWithProcessKey {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.processor_time.cmp(&other.processor_time).reverse()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    New,
    Running,
    WaitingForMessage,
    WaitingForResource,
    AskedToTerminate,
    Crashed
}

#[repr(C)]
pub struct ProcessDescriptor {
    process: ProcessBox,

    stack_overflow_guard : [u8; 4096],

    // 1 page frame
    stack : [u8; 4096],

    guard : [u8; 100],

    mailbox: VecDeque<Message>,

    children: Vec<u64>,

    state: ProcessState,

    registers: ProcessRegisters,

    maximum_execution_time : u64,

    execution_start_time : u64
}

#[derive(Copy, Clone, Debug)]
pub struct ProcessRegisters {
    pub instruction_pointer: u64,

    pub stack_pointer: u64,

    pub cpu_flags: u64,
}

impl ProcessDescriptor {
    fn new(process: ProcessBox, existing_count : u64) -> Self {
        let mailbox: VecDeque<Message> = VecDeque::new();
        let children: Vec<u64> = Vec::new();
        let state = ProcessState::New;
        let stack_overflow_guard = [0 as u8; 4096];
        let stack = [0 as u8; 4096];
        let guard = [0 as u8; 100];
        let maximum_execution_time = 1000 / existing_count;

        // process function will be called directly and those values will be populated after interrupt
        let registers = ProcessRegisters {
            instruction_pointer: 0,
            stack_pointer : 0,
            cpu_flags: 0,
        };

        let execution_start_time = 0;

        ProcessDescriptor {
            process,
            stack_overflow_guard,
            stack,
            guard,
            mailbox,
            children,
            state,
            registers,
            maximum_execution_time,
            execution_start_time
        }
    }

    pub fn set_execution_start_time(&mut self, time : u64) {
        self.execution_start_time = time;
    }

    pub fn create_guard(&mut self) {
        use memory::paging;
        use memory::frame::Frame;
        let table = paging::p4_table();
        unsafe { table.unmap_page(Frame::from_address(&self.stack_overflow_guard as *const _ as usize)) };
    }

    pub fn description(&self) -> &'static str {
        self.process.description()
    }

    pub fn registers(&self) -> &ProcessRegisters {
        &self.registers
    }

    pub fn state(&self) -> &ProcessState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ProcessState {
        &mut self.state
    }

    pub fn stack_address(&self) -> u64 {
        (&self.stack as *const _ as u64)
    }

    pub fn run_process(&mut self) -> () {
        self.state = ProcessState::WaitingForMessage;

        if let Some(message) = self.mailbox.pop_front() {

            if message.is::<Terminate>() {
                self.state = ProcessState::AskedToTerminate;
            }
            else {
                self.state = ProcessState::Running;
                self.process.process_message(message);
                self.state = ProcessState::WaitingForMessage;
            }
        }

        // this function must never exit. After reaching this line the process has finished processing message
        // and will be examined by executor on next scheduling iteration.
        loop { }
    }
}