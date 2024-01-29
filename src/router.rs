use crate::msg::Msg;
use core::convert::TryInto;
use freertos_rust::{
    Duration, FreeRtosError, FreeRtosUtils, InterruptContext, Queue, Task, TaskPriority,
};

const NUM_TASKS: usize = 4; // TODO: somehow make the number of tasks come from outside this file so the user can configure it
const QUEUE_DEPTH: usize = 5; // TODO: this used to be a const generic but moved it here till the NUM_TASKS is solved. Could also be task-specific.
const MAX_MESSAGES: usize = 10; // TODO: since `core::mem::variant_count()` isn't stable / const yet I don't know a good way to allocate an array that matches the number of enum variants

static mut ROUTER: Option<Router<NUM_TASKS>> = None;

pub type Route = Router<NUM_TASKS>;

#[allow(clippy::module_name_repetitions)]
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
        F: FnOnce(Task) + Send + 'static,
    {
        assert!(self.count < NUM_TASKS);

        let mut t = Task::new()
            .name(name)
            .stack_size(stack_size)
            .priority(TaskPriority(priority))
            .start(task)
            .unwrap();

        let q = Queue::new(QUEUE_DEPTH).unwrap();
        self.queues[self.count] = Some(q);
        self.count += 1;

        t.set_id(self.count.try_into().unwrap()); // Intentionally 1-based

        self
    }

    pub fn start(self) -> ! {
        assert_eq!(FreeRtosUtils::get_number_of_tasks(), NUM_TASKS);

        let router = Router::new(self.queues.map(Option::unwrap));
        unsafe { ROUTER = Some(router) };

        FreeRtosUtils::start_scheduler();
    }
}

pub struct Router<const N: usize> {
    queues: [Queue<Msg>; N],
    subs: [u32; MAX_MESSAGES],
}

impl<const N: usize> Router<N> {
    fn new(queues: [Queue<Msg>; N]) -> Self {
        assert!(N <= 32); // Only 32 bits are allocated to track which tasks are subscribed
        Self {
            queues,
            subs: [0; MAX_MESSAGES],
        }
    }

    pub fn subscribe(msg: Msg) {
        // TODO: is there a better way to specify this? I would rather subscribe using e.g. Msg::Blink or Msg::Blink(_).
        // Maybe impl Default on sub-messages and use a macro somehow? But that might add a lot of code space.

        let router = unsafe { ROUTER.as_mut().unwrap() };
        let msg_id = msg.id();
        let task_id = Self::get_current_task_id();
        router.subs[msg_id] |= 1 << task_id;
    }

    pub fn msg_send(msg: Msg) {
        let router = unsafe { ROUTER.as_ref().unwrap() };
        let msg_id = msg.id();
        for task_id in 0..N {
            if router.subs[msg_id] & (1 << task_id) != 0 {
                router.queues[task_id]
                    .send(msg, Duration::infinite())
                    .unwrap();
            }
        }
    }

    pub fn msg_send_isr(msg: Msg) {
        let router = unsafe { ROUTER.as_ref().unwrap() };
        let msg_id = msg.id();

        let mut isr_context = InterruptContext::new();
        for task_id in 0..N {
            if router.subs[msg_id] & (1 << task_id) != 0 {
                let result = router.queues[task_id].send_from_isr(&mut isr_context, msg);
                if matches!(result, Err(FreeRtosError::QueueFull)) {
                    panic!("Task {task_id} queue full");
                }
            }
        }
    }

    pub fn msg_rcv() -> Msg {
        let router = unsafe { ROUTER.as_ref().unwrap() };
        let task_id = Self::get_current_task_id();
        router.queues[task_id]
            .receive(Duration::infinite())
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn msg_rcv_nonblocking() -> Option<Msg> {
        let router = unsafe { ROUTER.as_ref().unwrap() };
        let task_id = Self::get_current_task_id();
        router.queues[task_id].receive(Duration::zero()).ok()
    }

    fn get_current_task_id() -> usize {
        // Task ID's are 1-based
        let id = Task::current().unwrap().get_id().unwrap() - 1;
        id.try_into().unwrap()
    }
}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationTickHook() {
    Route::msg_send_isr(Msg::Kick);
}
