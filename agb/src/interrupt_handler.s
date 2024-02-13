
@ An interrupt handler that simply acknowledges all interrupts
    .arm
    .global InterruptHandler
    .section .iwram.interrupt_handler, "ax", %progbits
    .align
InterruptHandler:

    .set IO_MEMORY_MAPPED_REGISTERS, 0x04000000
    .set OFFSET_INTERRUPT_ENABLED, 0x200

    mov r1, #IO_MEMORY_MAPPED_REGISTERS
    ldr r3, [r1, #OFFSET_INTERRUPT_ENABLED]!
    and r0, r3, r3, lsr #16 @ interrupts that are enabled AND triggered

    @ temporarily disable interrupts that were triggered here
    bic r2, r3, r0
    strh r2, [r1]


    @ r0: interrupts that are enabled AND triggered
    @ r1: #IO_MEMORY_MAPPED_REGISTERS + #OFFSET_INTERRUPT_ENABLED
    @ r3: Original contents of enabled interrupts    

    @ acknowledge interrupts
    strh r0, [r1, #2]

    .set OFFSET_BIOS_INTERRUPT_ACKNOWLEDGE, -0x8

    @ acknowledge bios interrupts
    sub r1, r1, #OFFSET_INTERRUPT_ENABLED
    ldrh r2, [r1, #OFFSET_BIOS_INTERRUPT_ACKNOWLEDGE]
    orr r2, r2, r0
    strh r2, [r1, #OFFSET_BIOS_INTERRUPT_ACKNOWLEDGE]

    @ r0: interrupts that are enabled AND triggered
    @ r1: #IO_MEMORY_MAPPED_REGISTERS
    @ r3: Original contents of enabled interrupts

    .set OFFSET_INTERRUPT_MASTER_ENABLE, 0x208

    @ clear interrupt master enable
    add r1, r1, #OFFSET_INTERRUPT_MASTER_ENABLE
    mov r2, #0
    swp r2, r2, [r1]

    @ r0: interrupts that are enabled AND triggered
    @ r2: old interrrupt master enable
    @ r3: Original contents of enabled interrupts

    @ push saved program status, old interrupt master enable, original enabled interrupts, and the link register
    mrs r1, spsr
    stmfd sp!, {{r1-r3, lr}}

    @ r0: interrupts that are enabled AND triggered

    .set PSR_MODE_MASK, 0x1F
    .set PSR_IRQ_DISABLE_MASK, 0x80
    .set PSR_MODE_SYSETM, 0x1F

    @ switch to system mode in the current program status register
    mrs r1, cpsr
    bic r1, r1, #(PSR_MODE_MASK | PSR_IRQ_DISABLE_MASK)
    orr r1, r1, #PSR_MODE_SYSETM
    msr cpsr, r1

    @ SYSTEM MODE

    push {{lr}}

    @ r0: interrupts that are enabled AND triggered

    @ call the rust interrupt handler with r0 set to the triggered interrupts
    ldr r1, =__RUST_INTERRUPT_HANDLER
    mov lr, pc
    bx r1


    pop {{lr}}


    @ NO MEANING TO ANY REGISTERS


    @ Clear the interrupt master enable
    mov r0, #IO_MEMORY_MAPPED_REGISTERS
    str r0, [r0, #OFFSET_INTERRUPT_MASTER_ENABLE]

    .set PSR_MODE_INTERRUPT, 0x12

    @ change back to interrupt mode
    mrs r1, cpsr
    bic r1, r1, #(PSR_MODE_MASK)
    orr r1, r1, #(PSR_MODE_INTERRUPT | PSR_IRQ_DISABLE_MASK)
    msr cpsr, r1

    @ r0: #IO_MEMORY_MAPPED_REGISTERS

    ldmfd sp!, {{r1-r3, lr}}
    msr spsr, r1
    str r2, [r0, #OFFSET_INTERRUPT_MASTER_ENABLE]!

    @ r0: #(IO_MEMORY_MAPPED_REGISTERS + OFFSET_INTERRUPT_MASTER_ENABLE)
    strh r3, [r0, #(OFFSET_INTERRUPT_ENABLED - OFFSET_INTERRUPT_MASTER_ENABLE)]

    bx lr @ return to bios
.pool


.section .iwram.program_counter
    .global agb_rs__program_counter
    .balign 4
agb_rs__program_counter:
    .word 0
