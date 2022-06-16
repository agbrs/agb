.include "src/asm_include.s"

.section .iwram.buffer_size
    .global agb_rs__buffer_size
    .balign 4
agb_rs__buffer_size:
    .word 0

agb_arm_func agb_rs__mixer_add
    @ Arguments
    @ r0 - pointer to the data to be copied (u8 array)
    @ r1 - pointer to the sound buffer (i16 array which will alternate left and right channels, 32-bit aligned)
    @ r2 - playback speed (usize fixnum with 8 bits)
    @ r3 - amount to modify the left channel by (u16 fixnum with 4 bits)
    @ stack position 1 - amount to modify the right channel by (u16 fixnum with 4 bits)
    @
    @ The sound buffer must be SOUND_BUFFER_SIZE * 2 in size = 176 * 2
    push {r4-r8}

    ldr r7, [sp, #20]        @ load the right channel modification amount into r7

    cmp r7, r3               @ check if left and right channel need the same modifications
    beq same_modification

modifications_fallback:
    orr r7, r7, r3, lsl #16   @ r7 now is the left channel followed by the right channel modifications.

    mov r5, #0                   @ current index we're reading from
    ldr r8, =agb_rs__buffer_size @ the number of steps left
    ldr r8, [r8]


1:
.rept 4
    add r4, r0, r5, asr #8    @ calculate the address of the next read from the sound buffer
    ldrsb r6, [r4]           @ load the current sound sample to r6
    add r5, r5, r2           @ calculate the position to read the next sample from

    ldr r4, [r1]             @ read the current value

    mla r4, r6, r7, r4       @ r4 += r6 * r7 (calculating both the left and right samples together)

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endr

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {r4-r8}
    bx lr

same_modification:
    @ check to see if this is a perfect power of 2
    @ r5 is a scratch register, r7 = r3 = amount to modify
    sub r5, r7, #1
    ands r5, r5, r7

    bne modifications_fallback @ not 0 means we need to do the full modification

    @ count leading zeros of r7 into r3
    mov r3, #0
1:
    add r3, r3, #1
    lsrs r7, r7, #1
    bne 1b

    mov r5, #0                   @ current index we're reading from
    ldr r8, =agb_rs__buffer_size @ the number of steps left
    ldr r8, [r8]

1:
.rept 4
    add r4, r0, r5, asr #8    @ calculate the address of the next read from the sound buffer
    ldrsb r6, [r4]            @ load the current sound sample to r6
    add r5, r5, r2           @ calculate the position to read the next sample from

    ldr r4, [r1]             @ read the current value

    lsl r6, r6, #16
    orr r6, r6, lsr #16
    add r4, r4, r6, lsl r3   @ r4 += r6 << r3 (calculating both the left and right samples together)

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endr

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {r4-r8}
    bx lr

agb_arm_end agb_rs__mixer_add

agb_arm_func agb_rs__mixer_add_stereo
    @ Arguments
    @ r0 - pointer to the data to be copied (u8 array)
    @ r1 - pointer to the sound buffer (i16 array which will alternate left and right channels, 32-bit aligned)
    @
    @ The sound buffer must be SOUND_BUFFER_SIZE * 2 in size = 176 * 2
    push {r4-r8}

    ldr r5, =0x00000FFF

    ldr r8, =agb_rs__buffer_size
    ldr r8, [r8]
1:
.rept 4
    ldrsh r6, [r0], #2        @ load the current sound sample to r6

    ldr r4, [r1]             @ read the current value

    @ This is slightly convoluted, but is mainly done for performance reasons. It is better
    @ to hit ROM just once and then do 3 really simple instructions then do 2 ldrsbs however annoying
    @ this is. Also, since all this code is in IWRAM and we never hit ROM otherwise, all accesses
    @ are sequential and exactly the size of the bus to ROM (16 bits), so hopefully this will be super fast.
    @
    @ The next 3 instructions set up the current value in r6 to be in the expected format
    @ 1 = 2s complement marks (so if negative, these are all 1s, if positive these are 0s)
    @ L = the left sample
    @ R = the right sample
    @ 0 = all zeros
    @ Split into bytes
    @
    @ At this point
    @                        r6 = | 1 | 1 | L | R | where the upper bytes are 1s if L is negative. No care about R
                         @ asr #8 | 1 | 1 | 1 | L | drop R off the right hand side
    and r7, r5, r6, asr #8 @ r7 = | 0 | 0 | 1 | L | exactly what we want this to be. The mask puts the 1 as 00001111 ready for the shift later
    lsl r6, r6, #24        @ r6 = | R | 0 | 0 | 0 | drop everything except the right sample
    orr r6, r7, r6, asr #8 @ r6 = | 1 | R | 1 | L | now we have it perfectly set up

    add r4, r4, r6, lsl #4  @ r4 += r6 << 4 (calculating both the left and right samples together)

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endr

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {r4-r8}
    bx lr

agb_arm_end agb_rs__mixer_add_stereo

.section .iwram
    .balign 4
constant_zero:
.rept 4
    .word 0
.endr

agb_arm_func agb_rs__init_buffer
    @ arguments:
    @ r0 = target buffer
    @ r1 = size in bytes (must be a multiple of 16)
    push {r4-r5}

    @ zero registers r3-r5
    ldr r2, =constant_zero
    ldm r2, {r3-r5,r12}

1:
    @ zero 4 words worth of the buffer
    stmia r0!, {r3-r5,r12}
    subs r1, r1, #(4 * 4)
    @ loop if we haven't zeroed everything
    bne 1b

    pop {r4-r5}
    bx lr
agb_arm_end agb_rs__init_buffer

agb_arm_func agb_rs__mixer_collapse
    @ Arguments:
    @ r0 = target buffer (i8)
    @ r1 = input buffer (i16) of fixnums with 4 bits of precision (read in sets of i16 in an i32)

    push {r4-r11}

CONST_0   .req r7
CONST_FF  .req r8
CONST_127 .req r9
TEMP      .req r10
SWAP_SIGN .req r11

    ldr CONST_0, =0
    ldr CONST_FF, =0xff
    ldr CONST_127, =127
    ldr SWAP_SIGN, =0x80808080

    ldr r2, =agb_rs__buffer_size @ loop counter
    ldr r2, [r2]
    mov r4, r2

@ The idea for this solution came from pimpmobile:
@ https://github.com/kusma/pimpmobile/blob/f2b2be49e806ca2a0d99cf91b3838d6d10f86b7d/src/pimp_mixer_clip_arm.S
@
@ The register should be 127 bigger then what you actually want, and we'll correct for that later. Hence the
@ add instructions in `load_sample`.
@
@ The idea behind this is in the bit patters of -128 and 127 which are 10000000 and 01111111 respectively,
@ and we want to clamp the value between them.
@
@ The first instruction calculates `-((sample + 128) >> 8)`. If sample is between -128 and 127, then
@ 0 <= sample + 128 <= 255 which means that shifting that right by 8 is 0. Hence the zero flag will be set, so
@ the `andne` instruction won't execute.
@
@ If the sample is outside of a signed 8 bit value, then `sample >> 8` will either be -1 or 1 (we assume that samples)
@ don't go too high, but the idea still works, so you can generalise this further if you want. This value is stored in TEMP
@
@ -1 has binary expansion (as a 32-bit integer) of all 1s and 1 of all zeros and then a 1.
@ So (-1 logical >> 24) gives 11111111 and (1 logical >> 24) gives 00000000 so register is clamped between these two values.
.macro clamp_s8 reg:req
    subs TEMP, CONST_0, \reg, asr #8
    andne \reg, CONST_FF, TEMP, lsr #24
.endm

.macro load_sample left_reg:req right_reg:req
    @ left_reg = *r1; r1++
    ldr \left_reg, [r1], #4

    mov \right_reg, \left_reg, lsl #16                 @ push the sample 16 bits first
    add \right_reg, CONST_127, \right_reg, asr #20     @ move right sample back to being the correct value
    add \left_reg, CONST_127, \left_reg, asr #20       @ now we only have the left sample

    clamp_s8 \left_reg                                 @ clamp the audio to 8 bit values
    clamp_s8 \right_reg
.endm

1:
.rept 4
    load_sample r3, r12

    load_sample r5, r6
    orr r3, r3, r5, lsl #8
    orr r12, r12, r6, lsl #8

    load_sample r5, r6
    orr r3, r3, r5, lsl #16
    orr r12, r12, r6, lsl #16

    load_sample r5, r6
    orr r3, r3, r5, lsl #24
    orr r12, r12, r6, lsl #24

    eor r3, r3, SWAP_SIGN
    eor r12, r12, SWAP_SIGN

    str r3, [r0, r4]       @ *(r0 + (r4 = SOUND_BUFFER_SIZE)) = r3
    str r12, [r0], #4      @ *r0 = r12; r0 += 4
.endr

    subs r2, r2, #16      @ r2 -= 16
    bne 1b               @ loop if not 0

    pop {r4-r11}
    bx lr
agb_arm_end agb_rs__mixer_collapse
