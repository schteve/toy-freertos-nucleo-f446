#![no_main]
#![no_std]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use cortex_m_rt::{entry, exception};
use freertos_rust::{FreeRtosAllocator, FreeRtosCharPtr, FreeRtosTaskHandle};
use nucleo_f446re::{button::Button, led::LedDigital, serial::SerialPort};
use panic_probe as _;
use router::RouterBuilder;
use stm32f4xx_hal::prelude::*;

mod msg;
mod router;
mod task_blink;
mod task_button;
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

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let user_led = LedDigital::new(gpioa.pa5);
    let user_button = Button::new(gpioc.pc13);
    let _timer = dp.TIM5.counter_us(&clocks); // counter_ms would cause an error because the prescaler would need to be too large
    let vcom = SerialPort::new(gpioa.pa2, gpioa.pa3, dp.USART2, &clocks);

    // Create tasks
    RouterBuilder::new()
        .install_task("blink", 512, 1, move || task_blink::task_blink(user_led))
        .install_task("button", 512, 1, move || {
            task_button::task_button(user_button)
        })
        .install_task("terminal", 512, 1, move || {
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
fn vApplicationMallocFailedHook() {}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationIdleHook() {}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {}

#[allow(non_snake_case)]
#[no_mangle]
fn vApplicationTickHook() {}
