/****************************************************************************
 *                                                          __              *
 *                ____ ___  ____ __  ______ ___  ____  ____/ /              *
 *               / __ `__ \/ __ `/ |/ / __ `__ \/ __ \/ __  /               *
 *              / / / / / / /_/ />  </ / / / / / /_/ / /_/ /                *
 *             /_/ /_/ /_/\__,_/_/|_/_/ /_/ /_/\____/\__,_/                 *
 *                                                                          *
 *         Copyright (c) 2008, Mukunda Johnson (mukunda@maxmod.org)         *
 *                                                                          *
 * Permission to use, copy, modify, and/or distribute this software for any *
 * purpose with or without fee is hereby granted, provided that the above   *
 * copyright notice and this permission notice appear in all copies.        *
 *                                                                          *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES *
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF         *
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR  *
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES   *
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN    *
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF  *
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.           *
 ****************************************************************************/

# 0 "mm_mas_arm.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "mm_mas_arm.S"
# 25 "mm_mas_arm.S"
@=============================================================
@ DEFINITIONS
@=============================================================

# 1 "mp_format_mas.inc" 1
# 23 "mp_format_mas.inc"
.ifndef MP_FORMAT_MAS_INC
.equ MP_FORMAT_MAS_INC, 1

@ MAS header structure..................................
.equ C_MAS_LEN ,0
.equ C_MAS_INSTN ,1
.equ C_MAS_SAMPN ,2
.equ C_MAS_PATTN ,3
.equ C_MAS_FLAGS ,4
.equ C_MAS_GV ,5
.equ C_MAS_SPEED ,6
.equ C_MAS_TEMPO ,7
.equ C_MAS_REP ,8

.equ C_MAS_CHANVOL ,12
.equ C_MAS_CHANPAN ,44
.equ C_MAS_ORDER ,76
.equ C_MAS_TABLES ,276

.equ C_FLAGS_GS ,1
.equ C_FLAGS_OS ,2
.equ C_FLAGS_SS ,3
.equ C_FLAGS_XS ,4
.equ C_FLAGS_DS ,5

.equ C_FLAGS_LS ,6

.equ C_FLAGS_X, (1<<3)

@ instrument struct.....................................
.equ C_MASI_GVOL ,0
.equ C_MASI_FADE ,1
.equ C_MASI_RANDVOL ,2
.equ C_MASI_DCT ,3
.equ C_MASI_NNA ,4
.equ C_MASI_ENVFLAGS ,5
.equ C_MASI_PAN ,6
.equ C_MASI_DCA ,7
.equ C_MASI_MAP ,8
.equ C_MASI_ENVELOPES,12


.equ C_MASIE_SIZE ,0
.equ C_MASIE_LSTART ,1
.equ C_MASIE_LEND ,2
.equ C_MASIE_SSTART ,3
.equ C_MASIE_SEND ,4
.equ C_MASIE_NODEC ,5
.equ C_MASIE_FILTER ,6
.equ C_MASIE_NODES ,8

.EQU ENVFLAG_A, 0b1000

@ sample structure......................................
.equ C_MASS_DV ,0
.equ C_MASS_PAN ,1
.equ C_MASS_FREQ ,2
.equ C_MASS_VIT ,4
.equ C_MASS_VID ,5
.equ C_MASS_VIS ,6
.equ C_MASS_VIR ,8
.equ C_MASS_GV ,7

.equ C_MASS_MSLID ,10

@ pattern structure.....................................
.equ C_MASP_NROWS ,0
.equ C_MASP_DATA ,1

@ sample structure......................................
.equ C_SAMPLE_LEN ,0
.equ C_SAMPLE_LOOP ,4
.equ C_SAMPLE_POINT ,12
.equ C_SAMPLE_DATA ,16

.equ C_SAMPLEN_LSTART,0
.equ C_SAMPLEN_LEN ,4
.equ C_SAMPLEN_FORMAT,8
.equ C_SAMPLEN_REP ,9
.equ C_SAMPLEN_POINT ,12
.equ C_SAMPLEN_DATA ,16

.equ C_SAMPLEC_DFREQ ,10


.endif
# 30 "mm_mas_arm.S" 2
# 1 "mp_mas_structs.inc" 1
# 23 "mp_mas_structs.inc"
@ Layer Information
@ -----------------

.equ MPL_TICK ,0
.equ MPL_ROW ,1
.equ MPL_POSITION ,2
.equ MPL_NROWS ,3
.equ MPL_GV ,4
.equ MPL_SPEED ,5
.equ MPL_ISPLAYING ,6
.equ MPL_BPM ,7
.equ MPL_INSTTABLE ,8
.equ MPL_SAMPTABLE ,12
.equ MPL_PATTTABLE ,16
.equ MPL_SONGADR ,20
.equ MPL_FLAGS ,24
.equ MPL_OLDEFFECTS ,25
.equ MPL_PATTJUMP ,26
.equ MPL_PATTJUMP_ROW,27
.equ MPL_FPATTDELAY ,28
.equ MPL_PATTDELAY ,29

.equ MPL_PLOOP_ROW ,30
.equ MPL_PLOOP_TIMES ,31
.equ MPL_PLOOP_ADR ,32
.equ MPL_PATTREAD ,36
.equ MPL_PLOOP_JUMP ,40
.equ MPL_VALID ,41

.equ MPL_TICKRATE ,42 @ 1.15 fixed point OR sample count
.equ MPL_SAMPCOUNT ,44 @ sample timing
.equ MPL_TICKFRAC ,44 @ vsync timing 0.16 fixed point

.equ MPL_MODE ,46
.equ mpl_reserved2 ,47
.equ MPL_MCH_UPDATE ,48
.equ MPL_VOLUME ,52
.equ mpl_reserved3 ,54
.equ MPL_SIZE ,56

@ active information
@--------------------

.equ MPV_PATTREAD_P ,4
.equ MPV_AFVOL ,8
.equ MPV_SAMPOFF ,9
.equ MPV_VOLPLUS ,10
.equ MPV_NOTEDELAY ,11
.equ MPV_PANPLUS ,12
.equ MPV_SIZE ,14

@ Module Channel
@ --------------

.EQU MCH_ALLOC, 0 @ ALLOCATED ACTIVE CHANNEL#
.EQU MCH_CFLAGS, 1 @ PATTERN COMRESSION FLAGS, called "maskvariable" in ITTECH.TXT
.EQU MCH_PANNING, 2
.EQU MCH_VOLCMD, 3 @ VOLUME COLUMN COMMAND
.EQU MCH_EFFECT, 4 @ EFFECT# } COMBINED
.EQU MCH_PARAM, 5 @ EFFECT PARAMETER }
.EQU MCH_FXMEM, 6 @ EFFECT MEMORY
.EQU MCH_NOTE, 7 @ TRANSLATED NOTE
.EQU MCH_FLAGS, 8 @ CHANNEL FLAGS
.EQU MCH_INST, 9 @ INSTRUMENT#

.EQU MCH_VIBDEP, 11
.EQU MCH_VIBSPD, 12
.EQU MCH_VIBPOS, 13
.EQU MCH_VOLUME, 14
.EQU MCH_CVOLUME, 15
.EQU MCH_PERIOD, 16
.EQU MCH_BFLAGS, 20
.EQU MCH_PNOTE, 22 @ PATTERN NOTE
.EQU MCH_MEMORY, 23 @ 15 bytes
.EQU MCH_SIZE, 40 @ sizeof( module_channel ) @ 2 extra bytes

@ Active Channel
@ --------------

.EQU MCA_PERIOD, 0 @ internal period
.EQU MCA_FADE, 4
.EQU MCA_ENVC_VOL, 6
.EQU MCA_ENVC_PAN, 8
.EQU MCA_ENVC_PIC, 10
.EQU MCA_AVIB_DEP, 12
.EQU MCA_AVIB_POS, 14
.EQU MCA_FVOL, 16
.EQU MCA_TYPE, 17
.EQU MCA_INST, 18
.EQU MCA_PANNING, 19
.EQU MCA_VOLUME, 20
.EQU MCA_SAMPLE, 21
.EQU MCA_PARENT, 22
.EQU MCA_FLAGS, 23
.EQU MCA_ENVN_VOL, 24
.EQU MCA_ENVN_PAN, 25
.EQU MCA_ENVN_PIC, 26
.EQU MCA_SFX, 27 @ can store this anywhere
.EQU MCA_SIZE, 28

@ Active Channel Flags
@ --------------------

.EQU MCAF_KEYON ,1 @ key is on... LOCKED
.EQU MCAF_FADE ,2 @ note-fade is activated
.EQU MCAF_START ,4 @ [re]start sample
.EQU MCAF_UPDATED ,8 @ already updated by pchannel routine
.EQU MCAF_ENVEND ,16 @ end of envelope
.EQU MCAF_VOLENV ,32 @ volume envelope enabled
.EQU MCAF_SUB ,64 @ sublayer.................locked..
.EQU MCAF_EFFECT ,128 @ subsublayer.............................LOCKED (mpEffect)

@ Active Channel Types
@ --------------------

.EQU ACHN_DISABLED ,0 @ LOCKED (multiple routines)
.EQU ACHN_RESERVED ,1 @ (can't be used [alloc channel])
.EQU ACHN_BACKGROUND ,2 @ LOCKED (alloc channel)
.EQU ACHN_FOREGROUND ,3
.EQU ACHN_CUSTOM ,4

@ Module Channel Flags
@ --------------------

.equ MF_START ,1
.equ MF_DVOL ,2
.equ MF_HASVCMD ,4
.equ MF_HASFX ,8
.equ MF_NEWINSTR ,16

.EQU MF_NOTEOFF ,64 @ LOCKED
.EQU MF_NOTECUT ,128 @ LOCKED

@.equ MF_NEWINSTR ,1 @ new instrument
@.EQU MF_DVOL ,2
@.EQU MF_START ,4
@.EQU MF_HASFX ,8
@.EQU MF_HASVCMD ,16
@.EQU MF_NOTEOFF ,64 @ LOCKED
@.EQU MF_NOTECUT ,128 @ LOCKED

@ Other Definitions
@ -----------------

.equ IT_NNA_CUT ,0 @ New note actions
.equ IT_NNA_CONT ,1
.equ IT_NNA_OFF ,2
.equ IT_NNA_FADE ,3

.equ IT_DCA_CUT ,0 @ Duplicate check actions
.equ IT_DCA_OFF ,1
.equ IT_DCA_FADE ,2

@ Misc Reference
@ --------------

@ BFLAGS:
@
@ nn...............new note action
@ pp...............panbrello waveform
@ tt...............tremolo waveform
@ vv...............vibrato waveform
@ dd...............duplicate check type
@ v................volume envelope enabled
@ t................tremor variable...
@ c................cut channel volume
@
# 31 "mm_mas_arm.S" 2
# 1 "mp_defs.inc" 1
# 26 "mp_defs.inc"
@ song 'mode' can be one of the following:
.equ MPP_PLAY_LOOP ,0
.equ MPP_PLAY_ONCE ,1
.equ MPP_PLAY_JINGLE ,2

@ other definitions


.struct 0
MM_GBA_SYSTEM_MODE: .space 4
MM_GBA_SYSTEM_MCH_COUNT: .space 4
MM_GBA_SYSTEM_ACH_COUNT: .space 4
MM_GBA_SYSTEM_MODCH: .space 4
MM_GBA_SYSTEM_ACTCH: .space 4
MM_GBA_SYSTEM_MIXCH: .space 4
MM_GBA_SYSTEM_MIXMEM: .space 4
MM_GBA_SYSTEM_WAVEMEM: .space 4
MM_GBA_SYSTEM_SOUNDBANK: .space 4
MM_GBA_SYSTEM_SIZE:

.equ SAMPFRAC, 12 @ # of bits used in fractional part of sample reading


.equ MP_SCHANNELS ,4

@ callback parameters

.equ MMCB_SONGREQUEST ,0x1A @ nds9
.equ MMCB_SAMPREQUEST ,0x1B @ nds9
.equ MMCB_DELETESONG ,0x1C @ nds9
.equ MMCB_DELETESAMPLE ,0x1D @ nds9

@.equ MPCB_SAMPMEMORY ,0x1E @ ---
@.equ MPCB_SONGMEMORY ,0x1F @ ---
.equ MMCB_BANKMEMORY ,0x1E @ nds9

.equ MPCB_SONGMESSAGE ,0x2A @ gba/nds7 song playback
.equ MPCB_SONGFINISHED ,0x2B @ gba/nds7

.struct 0
mms_rate: .space 4
mms_len: .space 4
mms_function: .space 4
mms_format: .space 4
mms_timer: .space 4
mms_manual: .space 1
mms_size:

.struct 0
mmrc_flags: .space 4
mmrc_memory: .space 4
mmrc_delay: .space 2
mmrc_rate: .space 2
mmrc_feedback: .space 2
mmrc_panning: .space 1

.equ MMRFS_MEMORY, 0
.equ MMRFS_DELAY, 1
.equ MMRFS_RATE, 2
.equ MMRFS_FEEDBACK, 3
.equ MMRFS_PANNING, 4
.equ MMRFS_LEFT, 5
.equ MMRFS_RIGHT, 6
.equ MMRFS_INVERSEPAN, 7
.equ MMRFS_NODRYLEFT, 8
.equ MMRFS_NODRYRIGHT, 9
.equ MMRFS_8BITLEFT, 10
.equ MMRFS_16BITLEFT, 11
.equ MMRFS_8BITRIGHT, 12
.equ MMRFS_16BITRIGHT, 13
.equ MMRFS_DRYLEFT, 14
.equ MMRFS_DRYRIGHT, 15


.equ MMRF_MEMORY, 1<<MMRFS_MEMORY
.equ MMRF_DELAY, 1<<MMRFS_DELAY
.equ MMRF_RATE, 1<<MMRFS_RATE
.equ MMRF_FEEDBACK, 1<<MMRFS_FEEDBACK
.equ MMRF_PANNING, 1<<MMRFS_PANNING
.equ MMRF_LEFT, 1<<MMRFS_LEFT
.equ MMRF_RIGHT, 1<<MMRFS_RIGHT
.equ MMRF_INVERSEPAN,1<<MMRFS_INVERSEPAN
.equ MMRF_NODRYLEFT, 1<<MMRFS_DRYLEFT
.equ MMRF_NODRYRIGHT,1<<MMRFS_DRYRIGHT
.equ MMRF_8BITLEFT, 1<<MMRFS_8BITLEFT
.equ MMRF_16BITLEFT, 1<<MMRFS_16BITLEFT
.equ MMRF_8BITRIGHT, 1<<MMRFS_8BITRIGHT
.equ MMRF_16BITRIGHT,1<<MMRFS_16BITRIGHT
.equ MMRF_DRYLEFT, 1<<MMRFS_DRYLEFT
.equ MMRF_DRYRIGHT, 1<<MMRFS_DRYRIGHT
# 32 "mm_mas_arm.S" 2
# 1 "mp_macros.inc" 1
# 60 "mp_macros.inc"
.macro ret0 @ arm7 requires pop/bx combo
 pop {r0}
 bx r0
.endm
.macro ret1
 pop {r1}
 bx r1
.endm
.macro ret2
 pop {r2}
 bx r2
.endm
.macro ret3
 pop {r3}
 bx r3
.endm


.macro jump7

 bl mpp_call_r7i



.endm

.macro jump2

 bl mpp_call_r2i



.endm

.macro fjump2 function
 ldr r2,=\function
 jump2
.endm

.macro jump1

 bl mpp_call_r1i



.endm



.macro break9
9: b 9b
.endm

.macro breakp
mov r11,r11
.endm

.macro GET_MIXCH reg




 ldr \reg,=mm_mixchannels
 ldr \reg, [\reg]

.endm



.macro PROF_START mode
# 145 "mp_macros.inc"
.endm

.macro PROF_END mode
# 164 "mp_macros.inc"
.endm

.macro __SECTION_IWRAM

 .section ".iwram", "ax", %progbits



.endm


.macro nocash_msg msg

 mov r12,r12
 b 9f
 .hword 0x6464
 .hword 0
 .string "\msg"
 .align 2
9:
.endm

.macro nocash_reset_clks

 nocash_msg %zeroclks%
.endm

.macro nocash_print_clks

 nocash_msg %lastclks%
.endm
# 33 "mm_mas_arm.S" 2


# 1 "mp_mixer_gba.inc" 1
# 26 "mp_mixer_gba.inc"
@ definitions

.equ MP_SAMPFRAC ,12

.equ MIXER_CHN_SRC ,0
.equ MIXER_CHN_READ ,4
.equ MIXER_CHN_VOL ,8
.equ MIXER_CHN_PAN ,9


.equ MIXER_CHN_FREQ ,12
.equ MIXER_CHN_SIZE ,16
# 36 "mm_mas_arm.S" 2
# 1 "swi_gba.inc" 1
# 23 "swi_gba.inc"
.equ SWI_DIVIDE, 0x06
# 37 "mm_mas_arm.S" 2







__SECTION_IWRAM

.arm
.align 2

.global mmAllocChannel
@********************************************************
mmAllocChannel:
@********************************************************


 push {r4,r5,r6}
 ldr r5,=mm_ch_mask
 ldr r5, [r5]
 ldr r1,=mm_achannels
 ldr r1, [r1]
 add r1, #MCA_FVOL
 mov r0, #0
 mov r2, #0x80000000
 mov r3, #255
 mov r6, #ACHN_BACKGROUND
 b .mppac_start
.mppac_skip:
 add r1, #MCA_SIZE
.mppac_next_test:
 cmp r5, #0
.mppac_next_notest:
 beq .mppac_finished
 add r0, #1
.mppac_start:
 movs r5, r5, lsr#1
 bcc .mppac_skip
 ldrh r4, [r1], #MCA_SIZE
 cmp r6, r4, lsr #8
 blt .mppac_next_test
 bgt .mppac_found

 cmp r2, r4, lsl#23
 bls .mppac_next_test
 mov r3, r0
 movs r2, r4, lsl#23

 b .mppac_next_test

.mppac_finished:
 mov r0, r3


.mppac_found:
 pop {r4,r5,r6}
 bx lr

.global mmReadPattern
@****************************************************
mmReadPattern:
@****************************************************
 push {r10-r12}
 ldr r11, [r8, #MPL_SONGADR]
 ldrb r12, [r11, #C_MAS_INSTN]
 ldrb r11, [r8, #MPL_FLAGS]
 mov r10, #1

 ldr r0,=mpp_channels
 ldr r9, [r0]
 ldr r7, [r8, #MPL_PATTREAD]
 ldr r0,=mpp_vars
 str r7, [r0, #MPV_PATTREAD_P]

 mov r1, #0

@----------------------------------------------------------------
readpattern:
@----------------------------------------------------------------

 mov r5, #0
 ldrb r3, [r7], #1
 movs r3, r3, lsl#32-7
 beq end_of_row
 rsb r3, r10, r3, lsr#32-7
 orr r1, r10, lsl r3
 mov r0, #MCH_SIZE
 mla r6, r3, r0, r9
 ldrcsb r2, [r7], #1
 strcsb r2, [r6, #MCH_CFLAGS]
 ldrccb r2, [r6, #MCH_CFLAGS]

 tst r2, #1
 beq no_note
 ldrb r0, [r7], #1
 cmp r0, #254
 strltb r0, [r6, #MCH_PNOTE]
 blt no_note
 orreq r5, #MF_NOTECUT
 orrgt r5, #MF_NOTEOFF
no_note:

 tst r2, #2
 beq no_instrument
 ldrb r0, [r7], #1
 tst r5, #MF_NOTEOFF|MF_NOTECUT
 bne no_instrument
 cmp r0, r12
 movgt r0, #0
 ldrb r3, [r6, #MCH_INST]
 cmp r0, r3
 beq same_instrument
 movs r3, r11, lsr#C_FLAGS_LS
 orrcs r5, #MF_START
 orr r5, #MF_NEWINSTR
same_instrument:
 strb r0, [r6, #MCH_INST]
no_instrument:

 tst r10, r2, lsr#3
 ldrcsb r0, [r7], #1
 strcsb r0, [r6, #MCH_VOLCMD]
no_vcmd:
 beq no_effect
 ldrb r0, [r7], #1
 ldrb r3, [r7], #1
 orr r0, r3, lsl#8
 strh r0, [r6, #MCH_EFFECT]
no_effect:
 orr r5, r2, lsr#4
 strb r5, [r6, #MCH_FLAGS]
 b readpattern
end_of_row:
 str r7, [r8, #MPL_PATTREAD]
 str r1, [r8, #MPL_MCH_UPDATE]
 pop {r10-r12}
 bx lr

.macro get_channel branch
 ldrb r0, [r7, #MCH_ALLOC]
 cmp r0, #255
 bge \branch
 ldr r6,=mm_achannels
 ldr r6, [r6]
 mov r1, #MCA_SIZE
 mla r6, r0, r1, r6
.endm

.global mmUpdateChannel_T0, mmUpdateChannel_TN
@***************************************************************************
mmUpdateChannel_T0:
@***************************************************************************
 push {lr}

 ldrb r5, [r7, #MCH_FLAGS]
 tst r5, #MF_START
 beq dont_start_channel

 tst r5, #MF_HASFX
 beq no_channel_effect
 ldrh r0, [r7, #MCH_EFFECT]
 and r1, r0, #0xFF






 tst r5, #MF_NEWINSTR
 bne start_channel
 cmp r1, #0x7
 beq glissando_affected

no_channel_effect:
 tst r5, #MF_NEWINSTR
 bne start_channel

 tst r5, #MF_HASVCMD
 beq start_channel
 ldrb r0, [r8, #MPL_FLAGS]
 movs r0, r0, lsr#C_FLAGS_XS
 ldrb r0, [r7, #MCH_VOLCMD]
 bcs xm_vcmd
it_vcmd:
 cmp r0, #193
 blt start_channel
 cmp r0, #202
 bgt start_channel
 b glissando_affected
xm_vcmd:
 cmp r0, #0xF0
 bge glissando_affected
 b start_channel
no_channel_vcmd:

glissando_affected:

 get_channel start_channel
# 245 "mm_mas_arm.S"
 bl mmChannelStartACHN
 bic r5, #MF_START
 strb r5, [r7, #MCH_FLAGS]
 b dont_start_channel

start_channel:

 ldr r2,=mpp_Channel_NewNote
 mov lr, pc
 bx r2
 get_channel mmUpdateChannel_TN_




 bl mmChannelStartACHN


 ldrb r0, [r6, #MCA_SAMPLE]
 subs r0, #1
 bcc no_sample_to_make_period

 ldrb r1, [r8, #MPL_FLAGS]
 lsrs r1, #C_FLAGS_SS
 bcs linear_periods

 ldr r3, [r8, #MPL_SONGADR]
 ldr r1, [r8, #MPL_SAMPTABLE]
 ldr r0, [r1, r0, lsl#2]
 add r0, r3

 ldrh r1, [r0, #C_MASS_FREQ]
 lsl r1, #2

 bl get_amiga_period
 b got_period
linear_periods:
 LDR R1,=IT_PitchTable
 LDR R0, [R1, R2, LSL#2]

got_period:


@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
 str r0, [r7, #MCH_PERIOD]
 ldrb r0, [r6, #MCA_FLAGS]
 orr r0, #MCAF_START
 strb r0, [r6, #MCA_FLAGS]
no_sample_to_make_period:


 b channel_started

dont_start_channel:

 get_channel mmUpdateChannel_TN___

channel_started:
 tst r5, #MF_DVOL
 beq dvol_skip
 ldrb r0, [r7, #MCH_INST]
 subs r0, #1
 bcc dvol_no_instrument

 ldr r2, [r8, #MPL_SONGADR]
 ldr r1, [r8, #MPL_INSTTABLE]
 ldr r0, [r1, r0, lsl#2]
 add r0, r2

 ldr r1, [r0, #C_MASI_NNA]
 ldrb r2, [r7, #MCH_BFLAGS]
 bic r2, #0b11000000
 orr r2, r1, lsl#6
 strb r2, [r7, #MCH_BFLAGS]

 ldrb r2, [r6, #MCA_FLAGS]
 bic r2, #MCAF_VOLENV
 tst r1, #ENVFLAG_A<<8
 orrne r2, #MCAF_VOLENV
 strb r2, [r6, #MCA_FLAGS]

 movs r1, r1, lsl#8+1
 movcs r1, r1, lsr#24
 strcsb r1, [r7, #MCH_PANNING]
dvol_no_instrument:

 ldrb r0, [r6, #MCA_SAMPLE]
 subs r0, #1
 bcc dvol_no_sample

 ldr r2, [r8, #MPL_SONGADR]
 ldr r1, [r8, #MPL_SAMPTABLE]
 ldr r0, [r1, r0, lsl#2]


 ldrh r1, [r0, r2]

 strb r1, [r7, #MCH_VOLUME]
 movs r1, r1, lsl#24-7
 mov r1, r1, lsr#24
 strcsb r1, [r7, #MCH_PANNING]
dvol_skip:
dvol_no_sample:

 tst r5, #MF_START|MF_DVOL
 beq dont_reset_volume

 ldrb r0, [r8, #MPL_FLAGS]
 tst r0, #C_FLAGS_X
 beq 1f
 tst r5, #MF_DVOL
 beq dont_reset_volume
1:
reset_volume:
 mov r0, #(1<<10)
 mov r1, #0
 mov r2, #0
 add r12, r6, #MCA_FADE
 stmia r12, {r0-r2}

 strh r1, [r6, #MCA_ENVN_VOL]
 strb r1, [r6, #MCA_ENVN_PIC]






 strb r1, [r7, #MCH_FXMEM]

 ldrb r1, [r6, #MCA_FLAGS]
 orr r1, #MCAF_KEYON
 bic r1, #MCAF_ENVEND | MCAF_FADE
 strb r1, [r6, #MCA_FLAGS]

dont_reset_volume:

 tst r5, #MF_NOTEOFF
 beq skip_noteoff

 ldrb r1, [r6, #MCA_FLAGS]
 bic r1, #MCAF_KEYON
 ldrb r0, [r8, #MPL_FLAGS]
 tst r0, #C_FLAGS_X
 orrne r1, #MCAF_FADE
 strb r1, [r6, #MCA_FLAGS]
skip_noteoff:

 tst r5, #MF_NOTECUT
 movne r0, #0
 strneb r0, [r7, #MCH_VOLUME]

 bic r5, #MF_START
 strb r5, [r7, #MCH_FLAGS]
 b mmUpdateChannel_TN_
@************************************************************
mmUpdateChannel_TN:
@************************************************************
 push {lr}

mmUpdateChannel_TN_:
 ldrb r0, [r7, #MCH_ALLOC]
 cmp r0, #255
 bge mmUpdateChannel_TN___
 ldr r6,=mm_achannels
 ldr r6, [r6]
 mov r1, #MCA_SIZE
 mla r6, r0, r1, r6

mmUpdateChannel_TN___:

 movge r6, #0

 ldr r5, [r7, #MCH_PERIOD]
 ldr r1,=mpp_vars
 mov r0, #0
 strb r0, [r1, #MPV_SAMPOFF]
 strb r0, [r1, #MPV_VOLPLUS]
 strb r0, [r1, #MPV_PANPLUS]
 strb r0, [r1, #MPV_NOTEDELAY]





 ldrb r0, [r7, #MCH_FLAGS]
 tst r0, #MF_HASVCMD
 ldrne r1,=mpp_Process_VolumeCommand
 movne lr, pc
 bxne r1






 ldrb r0, [r7, #MCH_FLAGS]
 tst r0, #MF_HASFX
 ldrne r1,=mpp_Process_Effect
 movne lr, pc
 bxne r1




 cmp r6, #0
 beq no_achn
 ldrh r0, [r7, #MCH_VOLUME]
 and r1, r0, #255
 mov r0, r0, lsr#8
 mul r0, r1, r0
 mov r0, r0, lsr#5
 strb r0, [r6, #MCA_VOLUME]

 ldr r1,=mpp_vars
 ldrsb r2, [r1, #MPV_VOLPLUS]
 adds r0, r2, lsl#3
 movmi r0, #0
 cmp r0, #129
 movcs r0, #128
 strb r0, [r1, #MPV_AFVOL]

 ldrb r0, [r1, #MPV_NOTEDELAY]
 cmp r0, #0
 beq channel_update_achn

 ldrb r0, [r6, #MCA_FLAGS]
 orr r0, #MCAF_UPDATED
 strb r0, [r6, #MCA_FLAGS]
 b no_achn

channel_update_achn:

 ldrb r0, [r7, #MCH_PANNING]
 strb r0, [r6, #MCA_PANNING]
 ldr r0, [r7, #MCH_PERIOD]
 str r0, [r6, #MCA_PERIOD]
 mov r0, #0
 strh r0, [r1, #MPV_PANPLUS]

 ldrb r0, [r6, #MCA_FLAGS]
 orr r0, #MCAF_UPDATED
 strb r0, [r6, #MCA_FLAGS]

 push {r4}
 ldrb r4, [r7, #MCH_ALLOC]
 ldr r1,=mpp_Update_ACHN_notest
 mov lr, pc
 bx r1

 pop {r4}
no_achn:
 pop {lr}
 bx lr

@***********************************************************
mmChannelStartACHN:
@***********************************************************
 ldrb r2, [r7, #MCH_BFLAGS+1]
 bic r2, #0b110
 strb r2, [r7, #MCH_BFLAGS+1]

 cmp r6, #0
 beq 1f

 mov r0, #ACHN_FOREGROUND
 strb r0, [r6, #MCA_TYPE]

 ldrb r0, [r6, #MCA_FLAGS]
 bic r0, #0b11000000
 ldr r1,=mpp_clayer
 ldrb r1, [r1]
 orr r0, r1, lsl#6
 orr r0, r10, r0, lsl#8
 strh r0, [r6, #MCA_PARENT]
 ldrb r0, [r7, #MCH_INST]
 strb r0, [r6, #MCA_INST]
1: ldreqb r0, [r7, #MCH_INST]
 subs r0, #1
 bcc invalid_instrument

 ldr r2, [r8, #MPL_SONGADR]
 ldr r1, [r8, #MPL_INSTTABLE]
 ldr r0, [r1, r0, lsl#2]
 add r0, r2

 ldrb r2, [r7, #MCH_PNOTE]

 ldrh r1, [r0, #C_MASI_MAP]
 tst r1, #0x8000
 beq full_notemap

 cmp r6, #0
 strneb r1, [r6, #MCA_SAMPLE]
 strb r2, [r7, #MCH_NOTE]
 bx lr

full_notemap:

 add r0, r2, lsl#1
 ldrh r2, [r0, r1]
 strb r2, [r7, #MCH_NOTE]
 cmp r6, #0
 mov r0, r2, lsr#8
 strneb r0, [r6, #MCA_SAMPLE]
 and r2, #255
invalid_instrument:
 bx lr
# 568 "mm_mas_arm.S"
.global mmGetPeriod
@***********************************************************
mmGetPeriod:
@***********************************************************

 ldrb r0, [r8, #MPL_FLAGS]
 lsrs r0, #C_FLAGS_SS
 bcs get_linear_period

get_amiga_period:
 adr r3, note_table_mod
 ldrb r0, [r3, r2]
 sub r3, #3*10
 ldrb r2, [r3, r2, lsr#2]





 ldr r3,=ST3_FREQTABLE
 ldrh r0, [r3, r0]

 mov r3, #0x00AB0
 orr r3, #0x20000
 mul r0, r3, r0
 mov r0, r0, lsr r2
 cmp r1, #0
 beq 1f
 swi SWI_DIVIDE<<16
1: bx lr

.global note_table_oct
.global note_table_mod
note_table_oct:
.byte 0,0,0,1,1,1,2,2,2,3,3,3,4,4,4,5,5,5, 6,6,6 ,7,7,7,8,8,8,9,9,9
note_table_mod:
.byte 0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0,2,4,6,8,10,12,14,16,18,20,22,0
.align 2

get_linear_period:
 ldr r1,=IT_PitchTable
 ldr r0, [r1, r2, lsl#2]
 bx lr

.pool
