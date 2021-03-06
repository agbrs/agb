    .arm
    .global __start
__start:
    b .Initialise

    @ Filled in by gbafix
    .fill 188, 1, 0

.Initialise:
    @ Set interrupt handler
    ldr r0, =InterruptHandlerSimple
    ldr r1, =0x03007FFC
    str r0, [r1]

    @ load main and branch
    ldr r0, =main
    bx  r0

.include "interrupt_simple.s"
