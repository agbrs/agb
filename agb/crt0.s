    .arm
    .global __start
__start:
    b .Initialise

    @ Filled in by gbafix
    .fill 188, 1, 0

    @ multiboot launch point
b .Initialise_mb

    .byte 0 @ boot mode, BIOS overwrites this value
    .byte 0 @ slave ID number
    .fill 26, 1, 0 @ unused?
    .word 0 @ joybus entrypoint

.Initialise_mb:
    swi 0x00250000

    @ Set interrupt handler
    ldr r0, =InterruptHandlerSimple
    ldr r1, =0x03007FFC
    str r0, [r1]

    b .CommonInit

.Initialise:
    @ Set interrupt handler
    ldr r0, =InterruptHandlerSimple
    ldr r1, =0x03007FFC
    str r0, [r1]

    @ copies ewram section in rom to ewram in ram
    ldr r0, =__ewram_rom_start                 @ load memory address storing start of data for ewram in rom
    ldr r1, =__ewram_data_start                @ load memory address storing location of ewram in ram
    ldr r2, =__ewram_rom_length_halfwords      @ load number of 16 bit values to copy
    swi 0x000B0000                             @ call interrupt CpuSet.
                                               @   r0: source
                                               @   r1: destination
                                               @   r2: length + size information
                                               @
                                               @ see: https://mgba-emu.github.io/gbatek/#swi-0bh-gbands7nds9dsi7dsi9---cpuset

.CommonInit:
    @ copies iwram section in rom to iwram in ram
    ldr r0, =__iwram_rom_start
    ldr r1, =__iwram_data_start
    ldr r2, =__iwram_rom_length_halfwords
    swi 0x000B0000

    @ put zero in both r0 and r1
    @ This corresponds to zero for argc and argv (which would technically be required for a c runtime)
    ldr r0, =0
    mov r1, r0

    @ load main and branch
    ldr r2, =main
    bx  r2

    @ loop if we end up here
1:
    b 1b
.pool

.include "interrupt_simple.s"
.include "src/sound/mixer/mixer.s"
