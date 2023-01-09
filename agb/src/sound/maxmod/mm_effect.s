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

# 0 "mm_effect.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "mm_effect.S"
# 25 "mm_effect.S"
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
# 26 "mm_effect.S" 2
# 1 "mp_mas.inc" 1
# 26 "mp_mas.inc"
@#ifdef SYS_NDS
@.equ MP_MCHANNELS ,16
@#else
@.equ MP_MCHANNELS ,32
@#endif

.extern mpp_vars
.extern mpp_pattread
.extern mpp_sm_handle

.extern mpp_resetchannels
.extern mpp_PlaySong
.extern mpp_processtick
# 27 "mm_effect.S" 2
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
# 28 "mm_effect.S" 2
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
# 29 "mm_effect.S" 2
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
# 30 "mm_effect.S" 2


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
# 33 "mm_effect.S" 2
# 45 "mm_effect.S"
.struct 0
MM_SFX_SOURCE: .space 4
MM_SFX_RATE: .space 2
MM_SFX_HANDLE: .space 2
MM_SFX_VOLUME: .space 1
MM_SFX_PANNING: .space 1
MM_SFX_SIZE:

.equ channelCount, 16
.equ releaseLevel, 200







 .BSS
 .ALIGN 2
 .GLOBAL mm_sfx_bitmask, mm_sfx_clearmask

mm_sfx_mastervolume: .space 4
mm_sfx_channels: .space 2*channelCount
mm_sfx_bitmask: .space 4
mm_sfx_clearmask: .space 4

mm_sfx_counter: .space 1







 .TEXT
 .THUMB
 .ALIGN 2




      .global mmResetEffects
      .thumb_func
mmResetEffects:

 mov r0, #0
 mov r1, #channelCount
 ldr r2,=mm_sfx_channels

1: strh r0, [r2]
 add r2, #2
 sub r1, #1
 bne 1b

 ldr r2,=mm_sfx_bitmask
 str r0, [r2]

 bx lr






      .thumb_func
mmGetFreeEffectChannel:

 ldr r0,=mm_sfx_bitmask
 ldr r0, [r0]
 mov r1, #1

.channel_search:
 lsr r0, #1
 bcc .found_channel
 add r1, #1
 b .channel_search

.found_channel:

 cmp r1, #channelCount+1
 bne .found_valid_channel
 mov r1, #0
.found_valid_channel:

 mov r0, r1
 bx lr






      .global mmEffect
      .thumb_func
mmEffect:
 push {lr}

 mov r1, #1
 lsl r1, #10
 ldr r2,=0x000080FF

 push {r0-r2}
 mov r0, sp
 bl mmEffectEx
 add sp, #12
 pop {r3}
 bx r3






      .global mmEffectEx
      .thumb_func
mmEffectEx:

 push {r4-r6, lr}

 mov r4, r0
 ldrh r5, [r4, #MM_SFX_HANDLE]

 cmp r5, #255
 bne 1f
 mov r5, #0
 b .got_handle

1: cmp r5, #0
 beq .generate_new_handle

 lsl r1, r5, #24
 lsr r1, #23
 sub r1, #2
 ldr r0,=mm_sfx_channels
 ldrb r0, [r0, r1]
 cmp r0, #0
 beq .got_handle

 mov r0, r5
 bl mmEffectCancel
 cmp r0, #0
 bne .got_handle

.generate_new_handle:

 bl mmGetFreeEffectChannel
 mov r5, r0
 beq .got_handle

 ldr r0,=mm_sfx_counter
 ldrb r1, [r0]
 add r1, #1
 strb r1, [r0]
 lsl r1, #8
 orr r5, r1
 lsl r5, #16
 lsr r5, #16

.got_handle:

 ldr r1,=mmAllocChannel
 bl mpp_call_r1
 cmp r0, #255
 bge .no_available_channels

 mov r6, r0

 cmp r5, #0
 beq 1f

 ldr r1,=mm_sfx_channels
 sub r2, r5, #1
 lsl r2, #24
 lsr r2, #24
 mov r3, #1
 lsl r3, r2
 lsl r2, #1
 add r1, r2
 add r2, r0, #1
 strb r2, [r1, #0]
 lsr r2, r5, #8
 strb r2, [r1, #1]

 ldr r1,=mm_sfx_bitmask
 ldr r2, [r1]
 orr r2, r3
 str r2, [r1]

1:

 ldr r1,=mm_achannels
 ldr r1, [r1]
 mov r2, #MCA_SIZE
 mul r2, r0
 add r1, r2

 mov r2, #releaseLevel
 strb r2, [r1, #MCA_FVOL]

 cmp r5, #0
 bne 1f
 mov r2, #ACHN_BACKGROUND
 b 2f
1: mov r2, #ACHN_CUSTOM
2:
 strb r2, [r1, #MCA_TYPE]
 mov r2, #MCAF_EFFECT
 strb r2, [r1, #MCA_FLAGS]

 GET_MIXCH r1
 mov r2, #MIXER_CHN_SIZE
 mul r2, r0
 add r3, r1, r2



 ldr r0,=mp_solution
 ldr r0, [r0]
 ldrh r1, [r4, #MM_SFX_SOURCE]
 lsl r1, #2
 add r1, #12
 ldr r1, [r0, r1]
 add r1, r0
 ldrh r2, [r1, #8+C_SAMPLEC_DFREQ]
 add r1, #8+C_SAMPLE_DATA
 str r1, [r3, #MIXER_CHN_SRC]

 ldrh r0, [r4, #MM_SFX_RATE]
 mul r2, r0
 lsr r2, #10-2
 str r2, [r3, #MIXER_CHN_FREQ]

 mov r1, #0
 str r1, [r3, #MIXER_CHN_READ]

 ldrb r0, [r4, #MM_SFX_VOLUME]

 ldr r1,=mm_sfx_mastervolume
 ldr r1, [r1]
 mul r0, r1
 lsr r0, #10

 strb r0, [r3, #MIXER_CHN_VOL]

 ldrb r0, [r4, #MM_SFX_PANNING]
 strb r0, [r3, #MIXER_CHN_PAN]
# 343 "mm_effect.S"
 mov r0, r5
 pop {r4-r6}
 ret1

.no_available_channels:
 mov r0, #0
 pop {r4-r6}
 ret1
# 359 "mm_effect.S"
      .thumb_func
mme_get_channel_index:

 lsl r1, r0, #24
 lsr r1, #24-1
 cmp r1, #0
 beq .invalid_handle
 cmp r1, #channelCount*2
 bgt .invalid_handle

 ldr r2,=mm_sfx_channels-2
 ldrh r3, [r2, r1]
 lsr r1, r3, #8
 lsr r2, r0, #8
 cmp r1, r2
 bne .invalid_handle

 lsl r3, #24
 lsr r3, #24
 sub r3, #1
 bx lr

.invalid_handle:
 mov r3, #0
 mvn r3, r3
 bx lr






      .global mmEffectActive
      .thumb_func
mmEffectActive:
 push {lr}
 bl mme_get_channel_index
 cmp r3, #0
 bge .active
 mov r0, #0
 pop {r3}
 bx r3

.active:
 mov r0, #1
 pop {r3}
 bx r3







      .thumb_func
mme_clear_channel:
 mov r1, #0
 ldr r2,=mm_sfx_channels
 lsl r0, #1
 strh r1, [r2, r0]

 mov r1, #1
 lsr r0, #1
 lsl r1, r0
 ldr r2,=mm_sfx_bitmask
 ldr r0, [r2, #4]
 orr r0, r1
 str r0, [r2, #4]
 ldr r0, [r2]
 bic r0, r1
 str r0, [r2]
 bx lr
# 439 "mm_effect.S"
      .global mmEffectVolume
      .thumb_func
mmEffectVolume:

 push {r1, lr}

 bl mme_get_channel_index
 pop {r1}
 bmi 1f

 ldr r0,=mm_sfx_mastervolume
 ldr r0, [r0]
 mul r1, r0
# 461 "mm_effect.S"
 lsr r1, #10



 mov r0, r3

 bl mmMixerSetVolume

1: ret0
# 478 "mm_effect.S"
      .global mmEffectPanning
      .thumb_func
mmEffectPanning:

 push {r1, lr}
 bl mme_get_channel_index
 pop {r1}
 bmi 1f

 mov r0, r3
 bl mmMixerSetPan

1: ret0






      .global mmEffectRate
      .thumb_func
mmEffectRate:

 push {r1, lr}
 bl mme_get_channel_index
 pop {r1}
 bmi 1f

 mov r0, r3
 bl mmMixerSetFreq

1: ret0






      .global mmEffectCancel
      .thumb_func
mmEffectCancel:

 push {r0, lr}

 bl mme_get_channel_index

 pop {r0}

 bmi 1f

 mov r1, #MCA_SIZE
 mul r1, r3
 ldr r2,=mm_achannels
 ldr r2, [r2]
 add r2, r1
 mov r1, #ACHN_BACKGROUND
 strb r1, [r2, #MCA_TYPE]
 mov r1, #0
 strb r1, [r2, #MCA_FVOL]

 lsl r0, #24
 lsr r0, #24
 sub r0, #1
 bl mme_clear_channel

 mov r1, #0
 mov r0, r3
 bl mmMixerSetVolume

 mov r0, #1
 ret1
1:
 mov r0, #0
 ret1






      .global mmEffectRelease
      .thumb_func
mmEffectRelease:

 push {r0, lr}

 bl mme_get_channel_index
 pop {r0}

 bmi 1f

 mov r1, #MCA_SIZE
 mul r1, r3
 ldr r2,=mm_achannels
 ldr r2, [r2]
 add r2, r1
 mov r1, #ACHN_BACKGROUND
 strb r1, [r2, #MCA_TYPE]

 lsl r0, #24
 lsr r0, #24
 sub r0, #1
 bl mme_clear_channel

1: ret0






      .global mmEffectScaleRate
      .thumb_func
mmEffectScaleRate:

 push {r1,lr}

 bl mme_get_channel_index
 pop {r1}

 bmi 1f

 mov r0, r3
 bl mmMixerMulFreq

1: ret0






      .global mmSetEffectsVolume
      .thumb_func
mmSetEffectsVolume:

 lsr r1, r0, #10
 beq 1f
 mov r0, #1
 lsl r0, #10

1: ldr r1,=mm_sfx_mastervolume
 str r0, [r1]
 bx lr






      .global mmEffectCancelAll
      .thumb_func
mmEffectCancelAll:

 push {r4-r7,lr}

 ldr r4,=mm_sfx_bitmask
 ldr r4, [r4]
 ldr r6,=mm_sfx_channels
 mov r5, #0


 lsr r4, #1
 bcc .mmeca_next
.mmeca_process:

 ldrb r7, [r6, r5]
 sub r7, #1
 bmi .mmeca_next

 mov r0, r7
 mov r1, #0
 bl mmMixerSetVolume

 ldr r0,=mm_achannels
 ldr r0, [r0]
 mov r1, #MCA_SIZE
 mul r1, r7
 add r0, r1
 mov r1, #ACHN_BACKGROUND
 strb r1, [r0, #MCA_TYPE]
 mov r1, #0
 strb r1, [r0, #MCA_FVOL]

.mmeca_next:
 add r5, #2
 lsr r4, #1
 bcs .mmeca_process
 bne .mmeca_next

 bl mmResetEffects

 POP {r4-r7}
 pop {r3}
 bx r3






      .global mmUpdateEffects
      .thumb_func
mmUpdateEffects:

 push {r4-r6,lr}

 ldr r4,=mm_sfx_bitmask
 ldr r4, [r4]
 ldr r6,=mm_sfx_channels
 mov r5, #0

 lsr r4, #1
 bcc .next_channel
.process_channel:

 ldrb r0, [r6, r5]
 sub r0, #1
 bmi .next_channel

 GET_MIXCH r1

 mov r2, #MIXER_CHN_SIZE
 mul r2, r0
 add r1, r2
# 712 "mm_effect.S"
 ldr r2, [r1, #MIXER_CHN_SRC]
 asr r2, #31
 beq .next_channel



 ldr r1,=mm_achannels
 ldr r1, [r1]
 mov r2, #MCA_SIZE
 mul r2, r0
 add r1, r2
 mov r0, #0
 strb r0, [r1, #MCA_TYPE]
 strb r0, [r1, #MCA_FLAGS]
 strb r0, [r6, r5]

.next_channel:
 add r5, #2
 lsr r4, #1
 bcs .process_channel
 add r5, #2
 lsr r4, #1
 bcs .process_channel
 bne .next_channel

 mov r4, #0
 mov r5, #1
 lsl r5, #32-channelCount
 ldr r6,=mm_sfx_channels

.build_new_bitmask:
 ldrb r0, [r6]
 add r6, #2
 cmp r0, #0
 beq 1f
 orr r4, r5
1: lsl r5, #1
 bne .build_new_bitmask

 lsr r4, #32-channelCount
 ldr r0,=mm_sfx_bitmask
 ldr r1, [r0]
 mov r2, r1
 eor r1, r4
 and r1, r2

 str r4, [r0]
 ldr r4, [r0, #4]
 orr r4, r1
 str r4, [r0, #4]

 pop {r4-r6,pc}

.pool

.end
