#![no_main]
#![no_std]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use cortex_m::asm;
use cortex_m_rt::{entry, exception};
use freertos_rust::*;
use panic_probe as _;

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    /*let h = Task::new().name("hello").stack_size(512).priority(TaskPriority(1)).start(|_this_task| {
        // Blink forever
        do_blink();
        loop {

        }
    }).unwrap();

    FreeRtosUtils::start_scheduler();*/

    loop {}
}

// Define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Alloc error, size: {}", layout.size());
}

#[allow(clippy::empty_loop)]
#[allow(non_snake_case)]
#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    asm::bkpt();
    loop {}
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
