use crate::{msg::Msg, router::Route};
use nucleo_f446re::button::Button;

#[derive(Clone, Copy)]
pub enum ButtonMsg {
    Released,
    Pressed,
}

// This task is responsible for reporting when the button is pressed
pub fn task_button(user_button: Button) -> ! {
    Route::subscribe(Msg::Kick);

    let mut pressed = false;
    loop {
        if let Msg::Kick = Route::msg_rcv() {
            let current = user_button.is_pressed();
            if current != pressed {
                pressed = current;
                if pressed {
                    Route::msg_send(Msg::Button(ButtonMsg::Pressed));
                } else {
                    Route::msg_send(Msg::Button(ButtonMsg::Released));
                }
            }
        }
    }
}
