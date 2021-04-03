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
    
    bx lr @ return to bios
