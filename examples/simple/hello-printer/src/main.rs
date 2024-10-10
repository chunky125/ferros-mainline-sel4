#![no_std]
#![no_main]

use ferros::*;
extern crate sel_claw;

use hello_printer::ProcParams;
use core::panic::PanicInfo;

#[cfg(feature = "panic_handler")]
mod panic;

#[no_mangle]
pub extern "C" fn _start(params: ProcParams) -> ! {
    debug_println!("hello from elf!");

    for i in 0..params.number_of_hellos {
        debug_println!("Hello elven world {}!", i);
    }

    unsafe {
        loop {
            sel_claw::seL4_Yield();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {

    loop {}
}


