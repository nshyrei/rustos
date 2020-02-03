use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::marker;
use core::cell;
use core::any::Any;


struct RemoveProcess {
    id : u64
}

struct StartProcess {}

struct CreateProcess {
    process_message : ProcessMessage
}

//impl Any for CreateProcess

type Message = Box<dyn Any>;

struct ProcessNode {

    process : ProcessMessage,

    mailbox : VecDeque<Message>,

    children : Vec<u64>
}

impl ProcessNode {

    fn new(process : ProcessMessage) -> Self {
        let mailbox : VecDeque<Message> = VecDeque::new();
        let children : Vec<u64> = Vec::new();

        ProcessNode {
            process,
            mailbox,
            children
        }
    }

}

pub struct ProcessReference {

    id : u64,

    executor : Rc<cell::RefCell<Executor>>
}

impl ProcessReference {

    fn post_message(&mut self, message : Message) {
        self.executor.borrow_mut().post_message(self.id, message)
    }

}

pub struct ProcessSystem {

}

pub struct ExecutorProcess {

    executor : Rc<cell::RefCell<Executor>>
}

impl ExecutorProcess {

    fn process(executor : &mut Executor, message : Message) {

        if message.is::<CreateProcess>() {
            let msg = message.downcast::<CreateProcess>().unwrap();

            executor.create_process(msg.process_message);
        }
        else if message.is::<RemoveProcess>() {
            let msg = message.downcast::<RemoveProcess>().unwrap();

            executor.remove_process_with_children(msg.id);
        }

        /*if let Some(msg) = message.downcast_ref::<CreateProcess>() {
            executor.create_process(msg.process_message);
        }

        if let Some(msg) = message.downcast_ref::<RemoveProcess>() {
            executor.remove_process_with_children(msg.id);
        }*/
    }

}

pub struct Executor {

    id_counter : u64,

    execution_line : VecDeque<u64>,

    existing : BTreeMap<u64, ProcessNode>,
}

impl Executor {

    fn new() -> Self {
        let id_counter = 1;
        let execution_line : VecDeque<u64> = VecDeque::new();
        let existing : BTreeMap<u64, ProcessNode> = BTreeMap::new();

        Executor {
            id_counter,
            execution_line,
            existing
        }
    }

    fn schedule_next(&mut self) {

        // Round robin algorithm: consecutively execute processes without any regard to priorities or round-trip time
        // pick one process to execute from execution line,
        // execute it and put it back into the queue
        if let Some(head_id) = self.execution_line.pop_front() {
            if let Some(head_node) = self.existing.get_mut(&head_id) {

                if let Some(mail) = head_node.mailbox.pop_front() {
                    let process_message = &head_node.process;

                    process_message(mail);
                }
            }

            self.execution_line.push_back(head_id);
        }
    }

    fn post_message(&mut self, id : u64, message : Message) {
        if let Some(process) = self.existing.get_mut(&id) {
            process.mailbox.push_back(message)
        }
    }

    fn remove_process(&mut self, id : u64) {
        self.existing.remove(&id);
    }

    fn remove_process_with_children(&mut self, id : u64) {
        if let Some(node) = self.existing.remove(&id) {
            for child_id in node.children {
                self.remove_process_with_children(child_id);
            }
        }
    }

    fn create_process(&mut self, process_message : ProcessMessage)  -> u64 {
        let node = ProcessNode::new(process_message);
        let id = self.id_counter;

        self.existing.insert(id, node);
        self.execution_line.push_back(id);
        self.id_counter += 1;

        id
    }

    fn fork(&mut self, parent_id : u64, process_message : ProcessMessage) -> u64 {
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
