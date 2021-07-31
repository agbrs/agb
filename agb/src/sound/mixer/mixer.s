.arm
.global agb_rs__mixer_add
.section .iwram, "ax"
.align
agb_rs__mixer_add:
    @ Arguments
    @ r0 - pointer to the data to be copied (u8 array)
    @ r1 - pointer to the sound buffer (i16 array)
    @ r2 - playback speed (usize fixnum with 8 bits)
    @ r3 - amount to modify the left channel by (u16 fixnum with 4 bits)
    @ stack position 1 - amount to modify the right channel by (u16 fixnum with 4 bits)
    @
    @ The sound buffer must be SOUND_BUFFER_SIZE * 2 in size = 176 * 2

    @ lr = amount to modify right channel by

    push {r4-r10, lr}

    @ load the right channel modification amount into lr
    ldr lr, [sp, #32]

    @ current write offset into the resulting buffer
    mov r12, #0
    mov r8, #352

    @ current index
    mov r5, #0

1:
    @ load the current sound buffer location
    mov r6, r1
    
    @ calculate the address of the next read form the sound buffer
    add r4, r0, r5, asr #8
    
    @ calculate the position to read the next step from
    add r5, r5, r2

    @ load the current buffer value (r6 being the current location, r12 being the offset)
    @ but pre-increment r6 by r12
    ldrh r9, [r6, r12]!

    @ load the current value we want to read
    ldrsb r10, [r4]

    @ increment the current write offset in the resulting buffer
    add r12, r12, #2

    @ check if we're done
    cmp r12, #352

    @ r7 = r10 * r3 + r9 = current sound value * left amount + previous buffer value
    mla r7, r10, r3, r9
    @ *(r6 + r8) = r7, r8 = 352 = offset for the right hand side
    strh r7, [r6], r8

    @ same for the left hand side (slightly confused here, but this is what was generated)
    ldrh r7, [r6]
    mla r4, r10, lr, r7
    strh r4, [r6]

    bne 1b

    pop {r4-r10, lr}
    bx lr
.pool

.arm
.global agb_rs__mixer_collapse
.section .iwram
.align
agb_rs__mixer_collapse:
    @ Arguments:
    @ r0 = target buffer (i8)
    @ r1 = input buffer (i16) of fixnums with 4 bits of precision

    mov r2, #0

1:
    @ r12 = *r1; r1++
    ldrsh r12, [r1], #2
    
    lsr r3, r12, #4     @ r3 = r12 >> 4

    cmn r12, #2048      @ compare r12 against -2048
    mvnlt r3, #127      @ r3 = -127 if r12 <= 2048

    cmp r12, #2048      @ compare r12 against 2048
    movge r3, #127      @ r3 = 127 if r12 >= 2048

    strb r3, [r0, -r2]  @ r2 counts down, so need a negative offset

    sub r2, r2, #1      @ r2 -= 1
    cmn r2, #352        @ compare r2 against -352

    bne 1b              @ loop if not equal

    bx lr
