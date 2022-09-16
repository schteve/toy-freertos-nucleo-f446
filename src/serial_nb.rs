use core::{cell::RefCell, fmt};
use cortex_m::{interrupt::Mutex, peripheral::NVIC};
use heapless::Deque;
use stm32f4xx_hal::{
    pac::{interrupt, Interrupt, USART2},
    prelude::*,
    serial::Tx,
};

const SERIAL_BUF_SIZE: usize = 200;
static mut GLOBAL: Mutex<RefCell<SerialNbGlobal<SERIAL_BUF_SIZE>>> =
    Mutex::new(RefCell::new(SerialNbGlobal::<SERIAL_BUF_SIZE>::new()));

struct SerialNbGlobal<const N: usize> {
    tx: Option<Tx<USART2>>,
    tx_buf: Deque<u8, N>,
}

impl<const N: usize> SerialNbGlobal<N> {
    const fn new() -> Self {
        Self {
            tx: None,
            tx_buf: Deque::new(),
        }
    }
}

pub struct SerialPortNb {
    _unconstructable: (),
}

impl SerialPortNb {
    pub fn create(tx: Tx<USART2>) -> Self {
        // TODO: some kind of protection against calling this twice? do for Router in any case.

        cortex_m::interrupt::free(|cs| {
            let mut global = unsafe { GLOBAL.borrow(cs).borrow_mut() };
            global.tx = Some(tx);
        });

        unsafe {
            NVIC::unmask(Interrupt::USART2);
        }

        Self {
            _unconstructable: (),
        }
    }

    #[allow(clippy::unused_self)] // We use self as a token to indicate the underlying resources
    pub fn write_byte(&mut self, byte: u8) {
        cortex_m::interrupt::free(|cs| {
            let mut global = unsafe { GLOBAL.borrow(cs).borrow_mut() };
            global
                .tx_buf
                .push_back(byte)
                .expect("Sender ran out of space");
            global.tx.as_mut().unwrap().listen();
        });
    }
}

impl fmt::Write for SerialPortNb {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut res = Ok(());
        cortex_m::interrupt::free(|cs| {
            let mut global = unsafe { GLOBAL.borrow(cs).borrow_mut() };
            res = s
                .bytes()
                .try_for_each(|b| global.tx_buf.push_back(b))
                .map_err(|_| fmt::Error);

            global.tx.as_mut().unwrap().listen();
        });
        res
    }
}

#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|cs| {
        let mut global = unsafe { GLOBAL.borrow(cs).borrow_mut() };
        if let Some(next) = global.tx_buf.pop_front() {
            global
                .tx
                .as_mut()
                .unwrap()
                .write(next)
                .expect("Failed to send");
        } else {
            global.tx.as_mut().unwrap().unlisten();
        }
    });
}
