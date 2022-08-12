use crate::{msg::Msg, router::Route, task_blink::BlinkMsg, task_button::ButtonMsg};

// This task is responsible for controlling the system.
// Right now it just listens for button presses and changes blinky accordingly.
pub fn task_controller() -> ! {
    Route::subscribe(Msg::Button(ButtonMsg::Released));

    loop {
        if let Msg::Button(x) = Route::msg_rcv() {
            match x {
                ButtonMsg::Released => Route::msg_send(Msg::Blink(BlinkMsg::Off)),
                ButtonMsg::Pressed => Route::msg_send(Msg::Blink(BlinkMsg::On)),
            }
        }
    }
}
