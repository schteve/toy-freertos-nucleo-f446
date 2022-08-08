use crate::task_blink::{BlinkMsg, BLINK_Q};
use freertos_rust::Duration;
use nucleo_f446re::button::Button;

pub fn task_button(user_button: Button) -> ! {
    let mut pressed = false;
    loop {
        let current = user_button.is_pressed();
        if current != pressed {
            pressed = current;
            if pressed {
                let q = unsafe { BLINK_Q.as_ref().unwrap() };
                q.send(BlinkMsg::Toggle, Duration::zero()).unwrap();
            }
        }
    }
}
