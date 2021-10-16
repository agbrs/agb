@ An interrupt handler that hands to an interrupt handler in rust
    .arm
    .global InterruptHandler
    .section .iwram, "ax"
    .align
InterruptHandler:
    mov r2, #0x04000000 @ interrupt enable register location
    add r2, #0x200

    mov r1, #0
    strh r1, [r2, #8]

    ldrh r1, [r2] @ load 16 bit interrupt enable to r1
    ldrh r3, [r2, #2] @ load 16 bit interrupt request to r3
    and r0, r1, r3 @ interrupts both enabled and requested

    @ change to system mode
    mrs r1, cpsr
    orr r1, r1, #0xD
    msr cpsr_c, r1

    @ call the rust interrupt handler with r0 set to the triggered interrupts
    ldr r1, =__RUST_INTERRUPT_HANDLER
    push {lr, r2}
    mov lr, pc
    bx r1
    pop {lr, r2}

    @ change back to interrupt mode
    mrs r1, cpsr
    bic r1, r1, #0xD
    msr cpsr_c, r1

    mov r1, #1
    strh r1, [r2, #8]

    strh r0, [r2, #2] @ store to interrupt request

    ldr r2, =0x03007FF8 @ load bios interrupt request location
    ldrh r1, [r2] @ load bios interrupt requests
    orr r1, r1, r0 @ or with enabled and requested interrupts
    strh r1, [r2] @ acknowlege bios requests

    bx lr @ return to bios
.pool


@ Swaps the value in r0 with the interrupt enable address
    .arm
    .global InterruptSwap
    .section .iwram, "ax"
    .align
InterruptSwap:
    mov r2, #0x04000000 @ interrupt enable register location
    add r2, #0x208
    swp r0, r0, [r2]
    bx lr
.pool
