use core::arch::global_asm;

global_asm!(concat!(
    include_str!("asm_include.s"),
    include_str!("entrypoint.s"),
    include_str!("interrupt_handler.s"),
    include_str!("sound/mixer/mixer.s"),
    include_str!("save/asm_routines.s")
));
