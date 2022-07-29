#![no_main]
#![no_std]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use cortex_m_rt::{entry, exception};
use freertos_rust::*;
use nucleo_f446re::led::LedDigital;
use panic_probe as _;
use stm32f4xx_hal::prelude::*;

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
    let mut user_led = LedDigital::new(gpioa.pa5);
    let mut timer = dp.TIM5.counter_us(&clocks); // counter_ms would cause an error because the prescalar would need to be too large

    let _h = Task::new()
        .name("blink")
        .stack_size(512)
        .priority(TaskPriority(1))
        .start(move || loop {
            user_led.toggle();
            timer.start(500.millis()).unwrap();
            nb::block!(timer.wait()).unwrap();
        })
        .unwrap();

    FreeRtosUtils::start_scheduler();
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
