use crate::{
    msg::Msg, router::Route, serial_nb::SerialPortNb, task_blink::BlinkMsg, task_button::ButtonMsg,
};
use core::{fmt::Write, str};
use nb::Error;
use nucleo_f446re::serial::SerialPort;
use stm32f4xx_hal::prelude::*;

pub fn task_terminal(vcom: SerialPort) -> ! {
    Route::subscribe(Msg::Kick);
    Route::subscribe(Msg::Button(ButtonMsg::Released));

    let SerialPort { tx, mut rx } = vcom;
    let mut serial = SerialPortNb::create(tx);

    let greeting = [
        "Welcome to the Nucleo command center terminal!",
        "You can:",
        "\tblinky on",
        "\tblinky off",
        "\tblinky toggle",
    ];
    serial.write_str("\r\n").unwrap();
    for s in greeting {
        serial.write_str(s).unwrap();
        serial.write_str("\r\n").unwrap();
    }

    let mut line = [0_u8; 20];
    let mut line_idx = 0;
    let mut clear = true;
    loop {
        if let Msg::Button(x) = Route::msg_rcv() {
            let str = match x {
                ButtonMsg::Released => "\r\nButton released!\r\n",
                ButtonMsg::Pressed => "\r\nButton pressed!\r\n",
            };
            serial.write_str(str).unwrap();
        }

        if clear {
            clear = false;
            line = [0_u8; 20];
            line_idx = 0;
            serial.write_str("\r\n? ").unwrap();
        }

        match rx.read() {
            Ok(c) => {
                serial.write_byte(c);

                if line_idx < line.len() {
                    if c == b'\r' || c == b'\n' {
                        // Process command after a line ending is received

                        // Rust str's don't play nice with null terminations so subslice to remove it
                        let null_pos = line.iter().position(|&c| c == b'\0').unwrap_or(line.len()); // Default to line length if no null present
                        let cmd = &line[0..null_pos];

                        match str::from_utf8(cmd) {
                            Ok(s) => {
                                match s {
                                    "blinky on" => {
                                        Route::msg_send(Msg::Blink(BlinkMsg::On));

                                        serial.write_str("\r\nYou light up the room.").unwrap();
                                    }
                                    "blinky off" => {
                                        Route::msg_send(Msg::Blink(BlinkMsg::Off));
                                        serial.write_str("\r\nIt is pitch black. You are likely to be eaten by a grue.").unwrap();
                                    }
                                    "blinky toggle" => {
                                        Route::msg_send(Msg::Blink(BlinkMsg::Toggle));

                                        serial.write_str("\r\nJust keep flipping the switch till something works out.").unwrap();
                                    }
                                    "" => (), // If sending two line endings in a row for example
                                    _ => {
                                        write!(serial, "\r\nOops, invalid command: {:?}", s)
                                            .unwrap();
                                    }
                                }
                            }
                            Err(_) => {
                                write!(serial, "Oops, invalid str: {:?}", line).unwrap();
                            }
                        }
                        clear = true;
                    } else if c == b'\x08' || c == b'\x7F' {
                        // Backspace
                        line_idx = line_idx.saturating_sub(1);
                        line[line_idx] = 0;
                    } else {
                        // Anything else
                        line[line_idx] = c;
                        line_idx += 1;
                    }
                } else {
                    serial.write_str("\r\nOops, line too long.").unwrap();
                    clear = true;
                }
            }
            Err(Error::WouldBlock) => (), // No character for us at the moment
            _ => {
                serial
                    .write_str("\r\nOops, something went wrong reading a character.")
                    .unwrap();
                clear = true;
            }
        }
    }
}
