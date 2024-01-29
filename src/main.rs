#![no_main]
#![no_std]
// For allocator
#![feature(alloc_error_handler)]

use core::{
    alloc::Layout,
    ffi::{c_char, CStr},
};
use cortex_m_rt::{entry, exception};
use freertos_rust::{FreeRtosAllocator, FreeRtosCharPtr, FreeRtosTaskHandle};
use nucleo_f446re::{button::Button, led::LedDigital, serial::SerialPort};
use panic_probe as _;
use router::RouterBuilder;
use stm32f4xx_hal::prelude::*;

mod msg;
mod router;
mod serial_nb;
mod task_blink;
mod task_button;
mod task_controller;
mod task_terminal;

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(180.MHz())
        .hclk(180.MHz())
        .freeze();

    let gpio_a = dp.GPIOA.split();
    let gpio_c = dp.GPIOC.split();

    let user_led = LedDigital::new(gpio_a.pa5);
    let user_button = Button::new(gpio_c.pc13);
    let _timer = dp.TIM5.counter_us(&clocks); // counter_ms would cause an error because the prescaler would need to be too large
    let vcom = SerialPort::new(gpio_a.pa2, gpio_a.pa3, dp.USART2, &clocks, 9600.bps()).unwrap();

    // Create tasks
    RouterBuilder::new()
        .install_task("blink", 512, 1, move |_task| {
            task_blink::task_blink(user_led)
        })
        .install_task("button", 512, 1, move |_task| {
            task_button::task_button(user_button)
        })
        .install_task("controller", 512, 2, move |_task| {
            task_controller::task_controller()
        })
        .install_task("terminal", 512, 1, move |_task| {
            task_terminal::task_terminal(vcom)
        })
        .start();
}

// Define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Alloc error, size: {}", layout.size());
}

#[allow(clippy::empty_loop)]
#[allow(non_snake_case)]
#[exception]
unsafe fn DefaultHandler(irqn: i16) -> ! {
    panic!("Unhandled IRQ: {}", irqn);
}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationMallocFailedHook() {
    panic!("Malloc failed");
}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationIdleHook() {}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    let name = unsafe { CStr::from_ptr(pcTaskName as *const c_char) };
    panic!("Stack overflow in task {name:?}");
}
