use crate::{msg::Msg, router::Route};
use nucleo_f446re::led::LedDigital;

#[derive(Clone, Copy)]
pub enum BlinkMsg {
    Off,
    On,
    Toggle,
}

pub fn task_blink(mut user_led: LedDigital) -> ! {
    Route::subscribe(Msg::Blink(BlinkMsg::Off));

    loop {
        match Route::msg_rcv() {
            Msg::Blink(x) => match x {
                BlinkMsg::Off => user_led.off(),
                BlinkMsg::On => user_led.on(),
                BlinkMsg::Toggle => user_led.toggle(),
            },
        }
    }
}
