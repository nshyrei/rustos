use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;
use core::ptr;
use hardware::x86_64::registers;

use crate::Message;
use crate::ProcessBox;

pub type ExecutorRef = Rc<cell::UnsafeCell<Executor>>;

pub struct ExecutorHelp {
    pub value : ptr::NonNull<Executor>
}

pub struct Executor {

    id_counter : u64,

    currently_executing : u64,

    execution_line : VecDeque<u64>,

    existing : BTreeMap<u64, ProcessDescriptor>,
}

impl Executor {

    pub fn new() -> Self {
        let id_counter = 0;
        let execution_line : VecDeque<u64> = VecDeque::new();
        let existing : BTreeMap<u64, ProcessDescriptor> = BTreeMap::new();

        Executor {
            id_counter,
            currently_executing : 0,
            execution_line,
            existing
        }
    }

    pub fn post_message(&mut self, id : u64, message : Message) {
        if let Some(process) = self.existing.get_mut(&id) {
            process.mailbox.push_back(message)
        }
    }

    pub(crate) fn remove_process_with_children(&mut self, id : u64) {
        if let Some(node) = self.existing.remove(&id) {
            for child_id in node.children {
                self.remove_process_with_children(child_id);
            }
        }
    }

    pub(crate) fn remove_process(&mut self, id : u64) {
        self.existing.remove(&id);
    }

    pub(crate) fn create_process(&mut self, process_message : ProcessBox) -> u64 {
        let node = ProcessDescriptor::new(process_message);
        let id = self.id_counter;

        self.existing.insert(id, node);
        self.execution_line.push_back(id);
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

    pub fn schedule_next(&mut self, current_process : ProcessRegisters) -> Option<ProcessRegisters> {
        // Round robin algorithm: consecutively execute processes without any regard to priorities or round-trip time
        // pick one process to execute from execution line,
        // execute it and put it back into the queue

        // current running process got interrupted, so we save its register state and put it in the back of the queue
        if let Some(existing_process) = self.existing.get_mut(&self.currently_executing) {
           // existing_process.registers = current_process;

            // put process to the back the queue for next round of scheduling
            self.execution_line.push_back(self.currently_executing);
        }

        // pick up next id from queue and associated descriptor for that id
        let queue_head = if let Some(head_id) = self.execution_line.pop_front() {

            self.existing.get_mut(&head_id).map(|p| (head_id, p))
        }
        else {
            None
        };

        // if there is something present in queue and process exists and its mailbox is not empty
        if let Some((head_id, head_node)) = queue_head {
None
            /*match head_node.state {
                ProcessState::Running => {

                    let result = head_node.registers;

                    self.currently_executing = head_id;

                    Some(result)
                },
                ProcessState::New => {
                    *//*if let Some(mail) = head_node.mailbox.pop_front() {

                        let process = head_node.process();

                        process.process_message(mail);
                    }*//*

                    // let code_pointer = head_node.process(); // howto extract proper function address

                    head_node.state = ProcessState::Running;

                    use core::ops::Deref;
                    //let stack_pointer = head_node.stack.deref() as *const _ as u64;

                    self.currently_executing = head_id;

                    Some(ProcessRegisters {
                        code_pointer : 0,

                        stack_pointer : 0,

                        cpu_flags : 0
                    })
                }
                _ => None
            }*/
        }
        else {
            None
        }
    }


    #[inline(always)]
    pub unsafe fn switch_to(process : ProcessRegisters) {
        registers::rflags_write(process.cpu_flags);
        registers::sp_write(process.stack_pointer as u32);
        registers::jump(process.code_pointer);
    }
}

enum ProcessState {
    New,
    Running,
    Finished
}

#[repr(C)]
struct ProcessDescriptor {

    process : ProcessBox,

    // 1 page frame
    //stack : Box<[u8; 4096]>,

    mailbox : VecDeque<Message>,

    children : Vec<u64>,

    //state : ProcessState,

    //registers : ProcessRegisters
}

#[derive(Copy, Clone)]
pub struct ProcessRegisters {

    pub code_pointer : u64,

    pub stack_pointer : u64,

    pub cpu_flags : u64
}

impl ProcessDescriptor {

    fn new(process : ProcessBox) -> Self {
        let mailbox : VecDeque<Message> = VecDeque::new();
        let children : Vec<u64> = Vec::new();
        let state = ProcessState::New;
        let stack = Box::new([0; 4096]);

        let registers = ProcessRegisters {
            code_pointer : 0,
            stack_pointer : 0,
            cpu_flags : 0
        };

        ProcessDescriptor {
            process,
            //stack,
            mailbox,
            children,
            //state,
            //registers
        }
    }

    fn process(&mut self) -> &mut ProcessBox {
        &mut self.process
    }
}