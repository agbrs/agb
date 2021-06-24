@ An interrupt handler that simply acknowledges all interrupts
    .arm
    .global InterruptHandlerSimple
    .align
InterruptHandlerSimple:
    ldr r2, =0x04000200 @ interrupt enable register location

    ldrh r1, [r2] @ load 16 bit interrupt enable to r1
    ldrh r3, [r2, #2] @ load 16 bit interrupt request to r3
    and r0, r1, r3 @ interrupts both enabled and requested
    strh r0, [r2, #2] @ store to interrupt request

    ldr r2, =0x03007FF8 @ load bios interrupt request location
    ldrh r1, [r2] @ load bios interrupt requests
    orr r1, r1, r0 @ or with enabled and requested interrupts
    strh r1, [r2] @ acknowlege bios requests

    @ change to system mode
    mrs r2, cpsr
    orr r2, r2, #0xD
    msr cpsr_c, r2

    @ call the rust interrupt handler with r0 set to the triggered interrupts
    ldr r1, =__RUST_INTERRUPT_HANDLER
    push {lr, r4-r9}
    mov lr, pc
    bx r1
    pop {lr, r4-r9}

    @ change back to interrupt mode
    mrs r2, cpsr
    bic r2, r2, #0xD
    msr cpsr_c, r2

    bx lr @ return to bios
.pool
