.equ SOUND_BUFFER_SIZE, 176

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

    orr r7, r7, r3, lsl #16   @ r7 now is the left channel followed by the right channel modifications.

    mov r5, #0               @ current index we're reading from
    mov r8, #SOUND_BUFFER_SIZE @ the number of steps left

1:
.macro mixer_add_loop
    add r4, r0, r5, asr #8    @ calculate the address of the next read from the sound buffer
    ldrsb r6, [r4]           @ load the current sound sample to r6
    add r5, r5, r2           @ calculate the position to read the next sample from

    ldr r4, [r1]             @ read the current value

    mla r4, r6, r7, r4       @ r4 += r6 * r7 (calculating both the left and right samples together)

    str r4, [r1], #4         @ store the new value, and increment the pointer
.endm

    mixer_add_loop
    mixer_add_loop
    mixer_add_loop
    mixer_add_loop

    subs r8, r8, #4          @ loop counter
    bne 1b                   @ jump back if we're done with the loop

    pop {r4-r8}
    bx lr
agb_arm_end agb_rs__mixer_add

.macro clamp_s8 reg:req
    cmn \reg, #127
    mvnlt \reg, #127

    cmp \reg, #128
    movgt \reg, #128
.endm

agb_arm_func agb_rs__mixer_collapse
    @ Arguments:
    @ r0 = target buffer (i8)
    @ r1 = input buffer (i16) of fixnums with 4 bits of precision (read in sets of i16 in an i32)

    mov r2, #SOUND_BUFFER_SIZE @ loop counter

1:
    @ r12 = *r1; r1++
    ldr r12, [r1], #4

    lsl r3, r12, #16        @ r3 is going to be the right sample, push r12 left 16 bits first
    asr r3, r3, #20         @ move r3 back to being the correct value
    mov r12, r12, asr #20   @ r12 = left sample

    clamp_s8 r12            @ clamp the audio to 8 bit values
    clamp_s8 r3

    strb r3, [r0, #SOUND_BUFFER_SIZE] @ *(r0 + SOUND_BUFFER_SIZE) = r3
    strb r12, [r0], #1                @ *r0 = r12; r0++

    subs r2, r2, #1      @ r2 -= 1
    bne 1b               @ loop if not 0

    bx lr
agb_arm_end agb_rs__mixer_collapse