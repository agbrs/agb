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

# 0 "mm_main_gba.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "mm_main_gba.S"
# 29 "mm_main_gba.S"
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
# 30 "mm_main_gba.S" 2
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
# 31 "mm_main_gba.S" 2
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
# 32 "mm_main_gba.S" 2
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
# 33 "mm_main_gba.S" 2
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
# 34 "mm_main_gba.S" 2







 .BSS
 .ALIGN 2






     .global mm_vblank_function
mm_vblank_function: .space 4
# 64 "mm_main_gba.S"
.section ".iwram", "ax", %progbits
.thumb
.align 2

.global mpp_call_r7i, mpp_call_r2i, mpp_call_r1i

 .thumb_func

mpp_call_r7i: bx r7


 .thumb_func

mpp_call_r2i: bx r2


 .thumb_func

mpp_call_r1i: bx r1


 .BSS
 .ALIGN 2






      .global mp_solution
mp_solution: .space 4

 .TEXT
 .THUMB
 .ALIGN 2






      .global mmInit
      .thumb_func
mmInit:
 push {lr}

 ldr r2,=mp_solution
 mov r1, #MM_GBA_SYSTEM_SOUNDBANK
 ldr r1, [r0,r1]
 str r1, [r2]

 ldr r2,=mm_achannels
 ldr r1, [r0,#MM_GBA_SYSTEM_ACTCH]
 str r1, [r2]
 ldr r1, [r0,#MM_GBA_SYSTEM_MODCH]
 str r1, [r2,#4]
 ldr r1, [r0,#MM_GBA_SYSTEM_MCH_COUNT]
 str r1, [r2,#8]
 ldr r1, [r0,#MM_GBA_SYSTEM_ACH_COUNT]
 str r1, [r2,#12]

 bl mmMixerInit @ initialize software/hardware mixer

 ldr r1,=mm_num_ach
 ldr r1,[r1]
 mov r0,#1
 lsl r0, r1
 sub r0,#1

 ldr r1,=mm_ch_mask
 str r0, [r1]

 ldr r0,=0x400
 bl mmSetModuleVolume
 ldr r0,=0x400
 bl mmSetJingleVolume
 ldr r0,=0x400
 bl mmSetEffectsVolume


 ldr r0,=0x400
 bl mmSetModuleTempo

 ldr r0,=0x400
 bl mmSetModulePitch

 bl mmResetEffects

 ret0






      .global mmSetVBlankHandler
      .thumb_func
mmSetVBlankHandler:

 ldr r1,=mm_vblank_function
 str r0, [r1]
 bx lr






      .global mmFrame
      .thumb_func
mmFrame:

 push {lr}
 push {r4-r7}

@ update effects

 ldr r7,=mmUpdateEffects
 bl _call_via_r7

@ update sub layer
@ sub layer has 60hz accuracy

 ldr r7,=mppUpdateSub
 bl _call_via_r7

@ update main layer and mix samples.
@ main layer is sample-accurate.

 ldr r0,=mpp_channels @ copy channels
 ldr r1,=mm_pchannels
 ldr r1,[r1]
 str r1, [r0]
 ldr r0,=mpp_nchannels @ copy #channels
 ldr r1,=mm_num_mch
 ldr r1,[r1]
 strb r1, [r0]
 ldr r0,=mpp_clayer @ layer=0 (main)
 mov r1, #0
 strb r1, [r0]

 ldr r0,=mmLayerMain @mpp_layerA @ copy layer pointer
 ldr r1,=mpp_layerp
 str r0, [r1]

 ldr r4,=mm_mixlen
 ldr r4,[r4]
     @ mixlen is divisible by 2

 ldrb r1, [r0, #MPL_ISPLAYING] @ check if main layer is active
 cmp r1, #0
 beq .mpf_no_mainlayer @ skip processing if disabled (and just mix samples)

.mpf_mix_advr:

 ldr r0,=mpp_layerp @ get layer
 ldr r0, [r0]

 mov r1, #MPL_TICKRATE @ get samples/tick
 ldrh r5, [r0, r1]

 mov r1, #MPL_SAMPCOUNT @ get sample count
 ldrh r6, [r0,r1]

 sub r5, r6 @ calc tickrate-counter
 cmp r5, #0
 bge 1f
 mov r5, #0
1: cmp r5, r4 @ > mixlen?
 blt .mpf_mix_adv @ no, mix and process tick
 b .mpf_mix @ yes, mix the rest of samples

.mpf_mix_adv:

 mov r1, #MPL_SAMPCOUNT @ reset sample counter
 mov r7, #0 @
 strh r7, [r0,r1] @
 sub r4, r5 @ subtract from #samples to mix

 PROF_START

 mov r0, r5
 ldr r7,=mmMixerMix @ mix samples
 bl _call_via_r7

 PROF_END 0

 ldr r7,=mppProcessTick
 bl _call_via_r7

 b .mpf_mix_advr @ process more samples
.mpf_mix:

@ add samples remaining to SAMPCOUNT
@ and mix more samples

 mov r1, #MPL_SAMPCOUNT
 add r6, r4
 strh r6, [r0, r1]
 mov r0, r4
 PROF_START
 ldr r1,=mmMixerMix
 bl _call_via_r1
 PROF_END 0

 pop {r4-r7}
 ret1 @ return to user

.mpf_no_mainlayer:

@ main layer isn't active,
@ mix full amount

 mov r0, r4

 PROF_START
 ldr r1,=mmMixerMix
 bl _call_via_r1
 PROF_END 0

 pop {r4-r7}
 ret1

.pool


.end
