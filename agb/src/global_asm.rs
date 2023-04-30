use core::arch::global_asm;

global_asm!(include_str!("crt0.s"));
global_asm!(include_str!("interrupt_handler.s"));
global_asm!(include_str!("sound/mixer/mixer.s"));
