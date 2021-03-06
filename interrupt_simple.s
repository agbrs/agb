@ An interrupt handler that simply acknowledges all interrupts
    .arm
    .global InterruptHandlerSimple
    .align
InterruptHandlerSimple:
    ldr r2, =0x04000200
    ldrh r1, [r2]
    ldrh r3, [r2, #2]
    and r0, r1, r3
    strh r0, [r2, #2]

    ldr r2, =0x03007FF8
    ldrh r1, [r2]
    orr r1, r1, r0
    strh r1, [r2]
    
    bx lr
