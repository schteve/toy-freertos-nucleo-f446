use crate::msg::Msg;
use freertos_rust::{
    Duration, FreeRtosBaseType, FreeRtosError, FreeRtosTaskHandle, FreeRtosUtils, Queue, Task,
    TaskPriority,
};

const NUM_TASKS: usize = 3; // TODO: somehow make the number of tasks come from outside this file so the user can configure it
const QUEUE_DEPTH: usize = 5; // TODO: this used to be a const generic but moved it here till the NUM_TASKS is solved. Could also be task-specific.

static mut ROUTER: Option<Router<NUM_TASKS>> = None;

pub type Route = Router<NUM_TASKS>;

pub struct RouterBuilder {
    queues: [Option<Queue<Msg>>; NUM_TASKS],
    queue_idx: usize,
}

impl RouterBuilder {
    pub fn new() -> Self {
        // TODO: If we're going to have a pool, allocate that somehow (static? just use freertos allocator?)

        const INIT: Option<Queue<Msg>> = None; // Workaround because Queue isn't Copy
        let queues = [INIT; NUM_TASKS];
        Self {
            queues,
            queue_idx: 0,
        }
    }

    pub fn install_task<F>(mut self, name: &str, stack_size: u16, priority: u8, task: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        assert!(self.queue_idx < NUM_TASKS);

        Task::new()
            .name(name)
            .stack_size(stack_size)
            .priority(TaskPriority(priority))
            .start(task)
            .unwrap();

        let q = Queue::new(QUEUE_DEPTH).unwrap();
        self.queues[self.queue_idx] = Some(q);
        self.queue_idx += 1;

        self
    }

    pub fn start(self) -> ! {
        let router = Router::<NUM_TASKS>(self.queues.map(Option::unwrap));
        unsafe { ROUTER = Some(router) };

        FreeRtosUtils::start_scheduler();
    }
}

pub struct Router<const N: usize>([Queue<Msg>; N]);

impl<const N: usize> Router<N> {
    pub fn msg_send(msg: Msg, to_task_id: usize) {
        assert!(to_task_id < N);

        let router = unsafe { ROUTER.as_ref().unwrap() };
        router.0[to_task_id].send(msg, Duration::zero()).unwrap();
    }

    pub fn msg_rcv() -> Msg {
        let router = unsafe { ROUTER.as_ref().unwrap() };
        let task_id = get_task_idx();
        router.0[task_id].receive(Duration::infinite()).unwrap()
    }
}

fn get_task_idx() -> usize {
    get_task_id().unwrap() as usize - 1 // Task ID is guaranteed to be 1+ since 0 is an error condition
}

fn get_task_id() -> Result<FreeRtosBaseType, FreeRtosError> {
    let task = Task::current().unwrap();
    let task_handle = unsafe { *(&task as *const _ as *const FreeRtosTaskHandle) }; // Yuck. There's no way to get at the task handle otherwise.
    let task_id = unsafe { freertos_rs_uxTaskGetTaskNumber(task_handle) };
    if task_id == 0 {
        Err(FreeRtosError::TaskNotFound)
    } else {
        Ok(task_id)
    }
}

extern "C" {
    pub fn freertos_rs_uxTaskGetTaskNumber(task_handle: FreeRtosTaskHandle) -> FreeRtosBaseType;
}
