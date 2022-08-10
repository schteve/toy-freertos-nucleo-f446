use freertos_rust::{Duration, Queue};
use nucleo_f446re::led::LedDigital;

#[derive(Clone, Copy)]
pub enum BlinkMsg {
    Off,
    On,
    Toggle,
}

pub static mut BLINK_Q: Option<Queue<BlinkMsg>> = None;

pub fn task_blink(mut user_led: LedDigital) -> ! {
    loop {
        let q = unsafe { BLINK_Q.as_ref().unwrap() };
        match q.receive(Duration::infinite()).unwrap() {
            BlinkMsg::Off => user_led.off(),
            BlinkMsg::On => user_led.on(),
            BlinkMsg::Toggle => user_led.toggle(),
        }
    }
}
