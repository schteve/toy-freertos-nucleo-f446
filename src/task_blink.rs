use crate::{msg::Msg, router::Route};
use nucleo_f446re::led::LedDigital;

#[derive(Clone, Copy)]
pub enum BlinkMsg {
    Off,
    On,
    Toggle,
}

// This task is responsible for blinking the LED as commanded
pub fn task_blink(mut user_led: LedDigital) -> ! {
    Route::subscribe(Msg::Blink(BlinkMsg::Off));

    loop {
        if let Msg::Blink(x) = Route::msg_rcv() {
            match x {
                BlinkMsg::Off => user_led.off(),
                BlinkMsg::On => user_led.on(),
                BlinkMsg::Toggle => user_led.toggle(),
            }
        }
    }
}
