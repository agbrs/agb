.macro mixer_add fn_name:req is_first:req
agb_arm_func \fn_name
    @ Arguments
    @ r0 - pointer to the data to be copied (u8 array)
    @ r1 - pointer to the sound buffer (i16 array which will alternate left and right channels, 32-bit aligned)
    @ r2 - playback speed (usize fixnum with 8 bits)
    @ r3 - amount to modify the left channel by (u16 fixnum with 4 bits)
    @ stack position 1 - amount to modify the right channel by (u16 fixnum with 4 bits)
    @ stack position 2 - the buffer_size (usize)
    @
    @ The sound buffer must be buffer_size * 2 in size
    push {{r4-r8}}

    ldr r7, [sp, #20]        @ load the right channel modification amount into r7
    ldr r8, [sp, #24]       @ load the buffer size into r8
    
    movs r8, r8             @ check that the buffer size isn't 0
    bne 1f

    pop {{r4-r8}}
    bx lr
1:

    cmp r7, r3               @ check if left and right channel need the same modifications
    beq 3f @ same modification

4:  @ modification fallback
    orr r7, r7, r3, lsl #16   @ r7 now is the left channel followed by the right channel modifications.

    mov r5, #0                   @ current index we're reading from

.macro add_one_sample
    add r4, r0, r5, asr #8    @ calculate the address of the next read from the sound buffer
    ldrsb r6, [r4]           @ load the current sound sample to r6
    add r5, r5, r2           @ calculate the position to read the next sample from

.ifc \is_first,true
    mul r4, r6, r7           @ r4 = r6 * r7 (calculating both the left and right samples together)
.else
    ldr r4, [r1]             @ read the current value
    mla r4, r6, r7, r4       @ r4 += r6 * r7 (calculating both the left and right samples together)
.endif

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endm
1:
.rept 4
    add_one_sample
.endr
.purgem add_one_sample
    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {{r4-r8}}
    bx lr

3: @ same modification
    @ check to see if this is a perfect power of 2
    @ r5 is a scratch register, r7 = r3 = amount to modify
    sub r5, r7, #1
    ands r5, r5, r7

    bne 4b @ not 0 means we need to do the full modification, jump to modification fallback

    @ count leading zeros of r7 into r3
    mov r3, #0
1:
    add r3, r3, #1
    lsrs r7, r7, #1
    bne 1b

    sub r3, r3, #1

    mov r5, #0                   @ current index we're reading from

.macro add_one_sample_same_modification
    add r4, r0, r5, asr #8    @ calculate the address of the next read from the sound buffer
    ldrsb r6, [r4]            @ load the current sound sample to r6
    add r5, r5, r2           @ calculate the position to read the next sample from

    lsl r6, r6, #16
    orr r6, r6, lsr #16

.ifc \is_first,true
    mov r4, r6, lsl r3       @ r4 = r6 << r3
.else
    ldr r4, [r1]             @ read the current value
    add r4, r4, r6, lsl r3   @ r4 += r6 << r3 (calculating both the left and right samples together)
.endif

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endm
1:
.rept 4
    add_one_sample_same_modification
.endr
.purgem add_one_sample_same_modification

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {{r4-r8}}
    bx lr

agb_arm_end \fn_name
.endm

mixer_add agb_rs__mixer_add false
mixer_add agb_rs__mixer_add_first true

.macro stereo_add_fn fn_name:req is_first:req
agb_arm_func \fn_name
    @ Arguments
    @ r0 - pointer to the data to be copied (u8 array)
    @ r1 - pointer to the sound buffer (i16 array which will alternate left and right channels, 32-bit aligned)
    @ r2 - volume to play the sound at
    @ r3 - the buffer size
    @
    @ The sound buffer must be SOUND_BUFFER_SIZE * 2 in size = 176 * 2
    push {{r4-r11}}

    ldr r5, =0x00000FFF

    mov r8, r3

.macro add_stereo_sample sample_reg:req
    ldrsh r6, [r0], #2        @ load the current sound sample to r6

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

.ifc \is_first,true
    mul \sample_reg, r6, r2
.else
    mla \sample_reg, r6, r2, \sample_reg     @ r4 += r6 * r2 (calculating both the left and right samples together)
.endif
.endm

1:
.ifc \is_first,true
.else
    ldmia r1, {{r9-r12}}       @ read the current values
.endif

    add_stereo_sample r9
    add_stereo_sample r10
    add_stereo_sample r11
    add_stereo_sample r12
.purgem add_stereo_sample

    stmia r1!, {{r9-r12}}         @ store the new value, and increment the pointer

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {{r4-r11}}
    bx lr

agb_arm_end \fn_name
.endm

stereo_add_fn agb_rs__mixer_add_stereo false
stereo_add_fn agb_rs__mixer_add_stereo_first true

agb_arm_func agb_rs__mixer_collapse
    @ Arguments:
    @ r0 = target buffer (i8)
    @ r1 = input buffer (i16) of fixnums with 4 bits of precision (read in sets of i16 in an i32)
    @ r2 = loop counter

    push {{r4-r11,lr}}

CONST_0   .req r7
CONST_128 .req r8
TEMP      .req r10
SWAP_SIGN .req r11

    ldr CONST_0, =0
    ldr CONST_128, =128
    ldr SWAP_SIGN, =0x80808080

    mov r4, r2

@ The idea for this solution came from pimpmobile:
@ https://github.com/kusma/pimpmobile/blob/f2b2be49e806ca2a0d99cf91b3838d6d10f86b7d/src/pimp_mixer_clip_arm.S
@
@ The register should be 127 bigger then what you actually want, and we'll correct for that later. Hence the
@ add instructions in `load_sample`.
@
@ The idea behind this is in the bit patters of -128 and 127 which are 10000000 and 01111111 respectively, -x = !x + 1 => !x = -x-1
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
    movne \reg, TEMP, lsr #24
.endm

.macro load_sample left_reg:req right_reg:req
    mov \right_reg, \left_reg, lsl #16                 @ push the sample 16 bits first
    add \right_reg, CONST_128, \right_reg, asr #20     @ move right sample back to being the correct value
    add \left_reg, CONST_128, \left_reg, asr #20       @ now we only have the left sample

    clamp_s8 \left_reg                                 @ clamp the audio to 8 bit values
    clamp_s8 \right_reg
.endm

1:
.rept 4
    ldmia r1!, {{r3,r5,r6,r9}}

    load_sample r3, r12

    load_sample r5, lr
    orr r3, r3, r5, lsl #8
    orr r12, r12, lr, lsl #8

    load_sample r6, lr
    orr r3, r3, r6, lsl #16
    orr r12, r12, lr, lsl #16

    load_sample r9, lr
    orr r3, r3, r9, lsl #24
    orr r12, r12, lr, lsl #24

    eor r3, r3, SWAP_SIGN
    eor r12, r12, SWAP_SIGN

    str r3, [r0, r4]       @ *(r0 + (r4 = SOUND_BUFFER_SIZE)) = r3
    str r12, [r0], #4      @ *r0 = r12; r0 += 4
.endr

    subs r2, r2, #16      @ r2 -= 16
    bne 1b               @ loop if not 0

    pop {{r4-r11,lr}}
    bx lr
agb_arm_end agb_rs__mixer_collapse
