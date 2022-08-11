use crate::{msg::Msg, router::Route, task_blink::BlinkMsg};
use nucleo_f446re::button::Button;

pub fn task_button(user_button: Button) -> ! {
    let mut pressed = false;
    loop {
        let current = user_button.is_pressed();
        if current != pressed {
            pressed = current;
            if pressed {
                Route::msg_send(Msg::Blink(BlinkMsg::Toggle));
            }
        }
    }
}
