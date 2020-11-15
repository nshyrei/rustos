use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::collections::binary_heap::PeekMut;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;
use core::ptr;
use core::ops;
use core::cmp;
use core::any::Any;
use core::fmt::Display;

use crate::process::{
    Message,
    Terminate,
    ProcessBox,
    Process
};

pub const TIME_QUANT : u64  = 1000;


pub struct Executor {
    id_counter: u64,

    execution_line: BinaryHeap<ProcessorTimeWithProcessKey>,

    new : VecDeque<u64>,

    existing: BTreeMap<u64, ProcessDescriptor>,
}

impl Executor {
    pub fn new() -> Self {
        let id_counter = 0;
        let execution_line: BinaryHeap<ProcessorTimeWithProcessKey> = BinaryHeap::new();
        let existing: BTreeMap<u64, ProcessDescriptor> = BTreeMap::new();
        let new : VecDeque<u64> = VecDeque::new();

        Executor {
            id_counter,
            execution_line,
            new,
            existing,
        }
    }

    /// Posts message to process
    /// # Arguments
    ///  `id` - process id
    ///  `message` - the message
    pub fn post_message(&mut self, id: u64, message: Message) {
        if let Some(process) = self.existing.get_mut(&id) {
            process.mailbox.push_back(message)
        }
    }

    pub fn currently_executing_mut(&mut self) -> Option<&mut ProcessDescriptor> {
        let execution_line = &self.execution_line;
        let existing = &mut self.existing;

        execution_line.peek().and_then(move |e| existing.get_mut(&e.process_key))
    }

    pub fn currently_executing(&self) -> Option<&ProcessDescriptor> {
        let execution_line = &self.execution_line;
        let existing = &self.existing;

        execution_line.peek().and_then(move |e| existing.get(&e.process_key))
    }

    pub fn currently_executing_id(&self) -> Option<u64> {

        self.execution_line.peek().map(|e| e.process_key)
    }

    fn remove_process_with_children(&mut self, id: u64) {
        if let Some(node) = self.existing.remove(&id) {
            for child_id in node.children {
                self.remove_process_with_children(child_id);
            }
        }
    }

    pub fn create_process(&mut self, process_message: ProcessBox) -> u64 {

        let mut node = ProcessDescriptor::new(process_message);
        //node.create_guard();

        let id = self.id_counter;

        node.process.set_id(id);
        self.existing.insert(id, node);
        self.new.push_back(id);

        self.id_counter += 1;

        id
    }

    pub fn fork(&mut self, parent_id : u64, process_message : ProcessBox) -> u64 {
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

        let currently_executing_opt = self.currently_executing_mut();

        if let Some(ProcessDescriptor { state : ProcessState::Running, registers, .. }) = currently_executing_opt {
            *registers = interrupted_process_registers;
        }
    }

    pub fn schedule_next(&mut self, current_time : u64) -> Option<&mut ProcessDescriptor> {
        for k in self.existing.keys() {
            if current_time == 12000 {
            let x = k;
            let a  = x;
        }
         
        }
        use hardware::x86_64::registers;
        let sp =  unsafe { registers::sp_read() };
        // remove any process that is terminated of crashed
        self.remove_terminated();

    for k in self.existing.keys() {
            let x = k;
            let a  = x;
        }

        let total_processes = self.execution_line.len() as u64;

        // set new processor time for currently executing process
        self.update_current_process_time(current_time, total_processes);

        // put new process (if we have any) into the execution queue
        self.put_new_process_to_execute(current_time, total_processes);

        // pick the top of the queue
        self.currently_executing_mut()
    }

    fn current_process_info(& self) -> Option<(&ProcessorTimeWithProcessKey, &ProcessDescriptor)> {
        let execution_line = &self.execution_line;
        let existing = & self.existing;

        let queue_top = execution_line.peek();

        //extract top of the queue and attach it to corresponding process descriptor
        queue_top.and_then(move |e| {
            let process_descriptor = existing.get(&e.process_key);

            process_descriptor.map(|d| (e, d))
        })
    }

    fn remove_terminated(&mut self) {
        let mut process_state = self.current_process_info();

        while let Some((time_description, ProcessDescriptor { state : ProcessState::AskedToTerminate, .. })) = process_state {
            let process_key = time_description.process_key;

            self.execution_line.pop();
            self.existing.remove(&process_key);

            process_state = self.current_process_info();
        }

        while let Some((time_description, ProcessDescriptor { state : ProcessState::Finished, .. })) = process_state {
            let process_key = time_description.process_key;

            self.execution_line.pop();
            self.new.push_back(process_key);

            process_state = self.current_process_info();
        }
    }

    fn update_current_process_time(&mut self, current_time : u64, total_processes: u64) {
        let queue_top = self.execution_line.peek_mut();

        // currently executing process sits at the top of the queue
        if let Some(mut process_time_descriptor) = queue_top {
            // time spent executing since last interrupt
            let processor_time = current_time - process_time_descriptor.interrupt_time;

            let total_processor_time = process_time_descriptor.processor_time + processor_time;

            let new_processor_time = if total_processor_time < process_time_descriptor.maximum_execution_time {
                // the bigger the processor time is, the lowest priority the process will get in the queue
                total_processor_time
            } else {
                // process has exhausted all its time and needs to make space for other processes
                // to make sure that it doesnt wind up into the front of the queue we add some artificial processor
                // time to it
                process_time_descriptor.maximum_execution_time / total_processes
            };

            process_time_descriptor.processor_time = new_processor_time;
            process_time_descriptor.interrupt_time = current_time;
        }
    }

    fn put_new_process_to_execute(&mut self, current_time : u64, total_processes : u64) {
        let has_message = self.new.front()
            .and_then(|id| self.existing.get(&id))
            .map(|e| e.mailbox.len() > 0)
            .unwrap_or(false);

        if has_message {
            // pick one (for now) new process and put it into the front of the queue
            let new_process = self.new.pop_front();

            new_process.map(|id| {
                let maximum_execution_time = if total_processes == 0 {
                    TIME_QUANT
                } else {
                    TIME_QUANT / total_processes
                };

                let new_entry = ProcessorTimeWithProcessKey {
                    processor_time: 0,
                    interrupt_time: current_time,
                    maximum_execution_time,
                    process_key: id,
                };

                self.execution_line.push(new_entry);
            });
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProcessorTimeWithProcessKey {
    processor_time: u64,

    interrupt_time : u64,

    maximum_execution_time : u64,

    process_key : u64
}

impl ProcessorTimeWithProcessKey {
    pub fn process_key(&self) -> u64 {
        self.process_key
    }
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
    Finished,
    Crashed
}

#[repr(C)]
pub struct ProcessDescriptor {
    // 1 page frame
    stack : [u8; 4096],

    process: ProcessBox,

    mailbox: VecDeque<Message>,

    children: Vec<u64>,

    state: ProcessState,

    registers: ProcessRegisters
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
        let stack = [0 as u8; 4096];

        // process function will be called directly and those values will be populated after interrupt
        let registers = ProcessRegisters {
            instruction_pointer: 0,
            stack_pointer : 0,
            cpu_flags: 0,
        };

        let execution_start_time = 0;

        ProcessDescriptor {
            process,
            stack,
            mailbox,
            children,
            state,
            registers
        }
    }

    pub fn create_guard(&mut self) {
        use memory::paging;
        use memory::frame::Frame;
        let table = paging::p4_table();
        //unsafe { table.unmap_page(Frame::from_address(&self.stack_overflow_guard as *const _ as usize)) };
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

    pub fn stack_head_address(&self) -> u64 {
        let chk = self.mailbox.len();

        // because stack grows into lesser address space we return last entry address
        // as a stack head
        &self.stack[4095] as *const _ as u64
    }

    pub fn run_process(&mut self) -> () {
        let chk = self.mailbox.len();
        self.state = ProcessState::WaitingForMessage;

        if let Some(message) = self.mailbox.pop_front() {

            if message.is::<Terminate>() {
                self.state = ProcessState::AskedToTerminate;
            }
            else {
                self.state = ProcessState::Running;
                self.process.process_message(message);
                self.state = ProcessState::Finished;
            }
        }

        // this function must never exit. After reaching this line the process has finished processing message
        // and will be examined by executor on next scheduling iteration.
        loop { }
    }
}