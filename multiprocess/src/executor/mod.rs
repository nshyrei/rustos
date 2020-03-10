use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;

use crate::Process;
use crate::Message;
use crate::ProcessBox;
use crate::process::ProcessRef;

pub type ExecutorRef = Rc<cell::RefCell<Executor>>;

pub struct Executor {

    id_counter : u64,

    execution_line : VecDeque<u64>,

    existing : BTreeMap<u64, ProcessNode>,
}

impl Executor {

    pub fn new() -> Self {
        let id_counter = 0;
        let execution_line : VecDeque<u64> = VecDeque::new();
        let existing : BTreeMap<u64, ProcessNode> = BTreeMap::new();

        Executor {
            id_counter,
            execution_line,
            existing
        }
    }

    pub(crate) fn post_message(&mut self, id : u64, message : Message) {
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
        let node = ProcessNode::new(process_message);
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

    fn schedule_next(&mut self) {
        // Round robin algorithm: consecutively execute processes without any regard to priorities or round-trip time
        // pick one process to execute from execution line,
        // execute it and put it back into the queue

        // pick up first id from queue and associated node for that id
        let node_info = if let Some(head_id) = self.execution_line.pop_front() {

            self.existing.remove(&head_id).map(|p| (head_id, p))
        }
        else {
            None
        };

        // if there is something present in queue and process exists and its mailbox is not empty
        if let Some((head_id, mut head_node)) = node_info {

            if let Some(mail) = head_node.mailbox.pop_front() {

                let process = head_node.process();

                process.process_message(mail);
            }

            // put executed process to the back the queue
            self.execution_line.push_back(head_id);
            self.existing.insert(head_id, head_node);
        }
    }
}

struct ProcessNode {

    process : ProcessBox,

    mailbox : VecDeque<Message>,

    children : Vec<u64>
}

impl ProcessNode {

    fn new(process : ProcessBox) -> Self {
        let mailbox : VecDeque<Message> = VecDeque::new();
        let children : Vec<u64> = Vec::new();

        ProcessNode {
            process,
            mailbox,
            children
        }
    }

    fn process(&mut self) -> &mut ProcessBox {
        &mut self.process
    }
}