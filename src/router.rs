use crate::msg::Msg;
use freertos_rust::{
    Duration, FreeRtosBaseType, FreeRtosError, FreeRtosTaskHandle, FreeRtosUBaseType,
    FreeRtosUtils, Queue, Task, TaskPriority,
};

const NUM_TASKS: usize = 3; // TODO: somehow make the number of tasks come from outside this file so the user can configure it
const QUEUE_DEPTH: usize = 5; // TODO: this used to be a const generic but moved it here till the NUM_TASKS is solved. Could also be task-specific.

static mut ROUTER: Option<Router<NUM_TASKS>> = None;

pub type Route = Router<NUM_TASKS>;

pub struct RouterBuilder {
    queues: [Option<Queue<Msg>>; NUM_TASKS],
    count: usize,
}

impl RouterBuilder {
    pub fn new() -> Self {
        // TODO: If we're going to have a pool, allocate that somehow (static? just use freertos allocator?)

        const INIT: Option<Queue<Msg>> = None; // Workaround because Queue isn't Copy
        let queues = [INIT; NUM_TASKS];
        Self { queues, count: 0 }
    }

    pub fn install_task<F>(mut self, name: &str, stack_size: u16, priority: u8, task: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        assert!(self.count < NUM_TASKS);

        let t = Task::new()
            .name(name)
            .stack_size(stack_size)
            .priority(TaskPriority(priority))
            .start(task)
            .unwrap();

        let q = Queue::new(QUEUE_DEPTH).unwrap();
        self.queues[self.count] = Some(q);
        self.count += 1;

        set_task_id(Some(t), self.count as u32); // Intentionally 1-based

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
        let task_id = get_task_id(None).unwrap() as usize - 1;
        router.0[task_id].receive(Duration::infinite()).unwrap()
    }
}

fn task_handle(task: Option<Task>) -> FreeRtosTaskHandle {
    let task = if let Some(t) = task {
        t
    } else {
        Task::current().unwrap()
    };
    unsafe { *(&task as *const _ as *const FreeRtosTaskHandle) } // Yuck. There's no way to get at the task handle otherwise.
}

fn get_task_id(task: Option<Task>) -> Result<FreeRtosBaseType, FreeRtosError> {
    let task_handle = task_handle(task);
    let task_id = unsafe { freertos_rs_uxTaskGetTaskNumber(task_handle) };
    if task_id == 0 {
        Err(FreeRtosError::TaskNotFound)
    } else {
        Ok(task_id)
    }
}

fn set_task_id(task: Option<Task>, value: FreeRtosUBaseType) {
    let task_handle = task_handle(task);
    unsafe { freertos_rs_vTaskSetTaskNumber(task_handle, value) };
}

extern "C" {
    pub fn freertos_rs_uxTaskGetTaskNumber(task_handle: FreeRtosTaskHandle) -> FreeRtosBaseType;
    pub fn freertos_rs_vTaskSetTaskNumber(
        task_handle: FreeRtosTaskHandle,
        value: FreeRtosUBaseType,
    );
}
