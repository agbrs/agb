@ An interrupt handler that simply acknowledges all interrupts
    .arm
    .global InterruptHandlerSimple
    .align
InterruptHandlerSimple:
    ldr r4, =0x04000200 @ interrupt enable register location

    ldrh r1, [r4] @ load 16 bit interrupt enable to r1
    ldrh r3, [r4, #2] @ load 16 bit interrupt request to r3
    and r0, r1, r3 @ interrupts both enabled and requested
    strh r0, [r4, #2] @ store to interrupt request

    ldr r2, =0x03007FF8 @ load bios interrupt request location
    ldrh r1, [r2] @ load bios interrupt requests
    orr r1, r1, r0 @ or with enabled and requested interrupts
    strh r1, [r2] @ acknowlege bios requests

    @ change to user mode
    mrs r2, cpsr
    orr r2, r2, #0xD
    msr cpsr_c, r2

    @ call the rust interrupt handler with r0 set to the triggered interrupts
    ldr r1, =__RUST_INTERRUPT_HANDLER
    push {lr, r4}
    adr lr, .IReturn
    bx r1
.IReturn:
    pop {lr, r4}

    @ change back to interuupt mode
    mrs r2, cpsr
    bic r2, r2, #0xD
    orr r2, r2, #0x92
    msr cpsr_c, r2

    bx lr @ return to bios
.pool
