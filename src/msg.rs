use crate::task_blink::BlinkMsg;

#[derive(Clone, Copy)]
pub enum Msg {
    Blink(BlinkMsg),
}
