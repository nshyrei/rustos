use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::any::Any;

struct ProcessNode {

    process : Process,

    mailbox : VecDeque<Box<dyn Any>>,

    children : Vec<ProcessNode>
}

impl ProcessNode {

    /*pub fn mailbox(&mut self) -> &mut VecDeque<Box<dyn Any>> {
        &mut self.mailbox
    }*/

}

pub struct Executor {

    id_counter : u64,

    execution_line : VecDeque<u64>,

    existing : BTreeMap<u64, ProcessNode>,

    executor_process : ProcessNode
}

impl Executor {

    fn schedule_next(&mut self) {

        // process system mail first
        // pick one system message

        if let Some(system_message) = self.executor_process.mailbox.pop_front() {
            /*match  system_message.downcast_ref::<RemoveProcess>() {

            };*/
        }

        // Round robin algorithm: consecutively execute processes without any regard to priorities or round-trip time
        // pick one process to execute from execution line,
        // execute it and put it back into the queue
        if let Some(head_id) = self.execution_line.pop_front() {
            if let Some(head_node) = self.existing.get_mut(&head_id) {

                if let Some(mail) = head_node.mailbox.pop_front() {
                    let process_message = head_node.process.process_message_function();

                    process_message(mail);
                }
            }



            self.execution_line.push_back(head_id);
        }
    }

    fn remove_process(&mut self, id : u64) {
        //self.mailboxes.remove(&id);

    }
}

pub type ProcessMessage = Box< dyn Fn(Box<dyn Any>) -> ()>;

pub struct Process {

    id : u64,

    executor : Rc<Executor>,

    process_message : ProcessMessage
}

impl Process {

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn process_message_function(&self) -> &ProcessMessage {
        &self.process_message
    }

    pub fn new(id : u64, executor : Rc<Executor>, process_message : ProcessMessage) -> Self {
        Process {
            id,
            executor,
            process_message
        }
    }
}

struct RemoveProcess {}
struct StartProcess {}
