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


    @ copies iwram section in rom to iwram in ram
    ldr r0, =__iwram_rom_start              @ load memory address storing start of data for iwram in rom
    ldr r1, =__iwram_data_start             @ load memory address storing location of iwram in ram
    ldr r2, =__iwram_rom_length_halfwords   @ load number of 16 bit values to copy
    swi 0x000B0000                          @ call interrupt CpuSet.
                                            @   r0: source
                                            @   r1: destination
                                            @   r2: length + size information
                                            @
                                            @ see: https://mgba-emu.github.io/gbatek/#swi-0bh-gbands7nds9dsi7dsi9---cpuset

    @ copies ewram section in rom to ewram in ram
    ldr r0, =__ewram_rom_start
    ldr r1, =__ewram_data_start
    ldr r2, =__ewram_rom_length_halfwords
    swi 0x000B0000


    @ put zero in both r0 and r1
    @ This corresponds to zero for argc and argv (which would technically be required for a c runtime)
    ldr r0, =0
    mov r1, r0

    @ load main and branch
    ldr r2, =main
    bx  r2
.pool

.include "interrupt_simple.s"
