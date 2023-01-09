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

# 0 "mm_mixer_gba.S"
# 0 "<built-in>"
# 0 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 0 "<command-line>" 2
# 1 "mm_mixer_gba.S"
# 25 "mm_mixer_gba.S"
@ DEFINITIONS

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
# 28 "mm_mixer_gba.S" 2
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
# 29 "mm_mixer_gba.S" 2
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
# 30 "mm_mixer_gba.S" 2

@ timer freq = 2^24 / mixfreq
@ mixlen ~= mixfreq * 0.01673
@ recommended mixing frequencies: 5734,7884,10512,13379,15768,18157,21024,26758,31536,36314,40137,42048
@ other mixing frequencies may cause clicks
@ mixlen must be divisible by 2

@==================================================================
@ GLOBAL SYMBOLS
@==================================================================


 .global mmVBlank
 .type mmVBlank STT_FUNC
 .global mmMixerMix
 .type mmMixerMix STT_FUNC

 .global mmMixerSetSource
 .type mmMixerSetSource STT_FUNC
 .global mmMixerSetRead
 .type mmMixerSetRead STT_FUNC
 .global mmMixerSetFreq
 .type mmMixerSetFreq STT_FUNC
 .global mmMixerStopChannel
 .type mmMixerStopChannel STT_FUNC
 .global mmMixerInit
 .type mmMixerInit STT_FUNC
 .global mmMixerChannelActive
 .type mmMixerChannelActive STT_FUNC
 .global mmMixerMulFreq
 .type mmMixerMulFreq STT_FUNC
 .global mmMixerSetVolume
 .type mmMixerSetVolume STT_FUNC
 .global mmMixerSetPan
 .type mmMixerSetPan STT_FUNC


 .global mm_mixlen
 .global mm_bpmdv
 .global mp_mix_seg
 .global mp_writepos
 .global mm_mixchannels

@===============================================
@ more definitions
@===============================================

.EQU REG_SOUNDCNT_L, 0x4000080
.EQU REG_SOUNDCNT_H, 0x4000082
.EQU REG_SOUNDCNT_X, 0x4000084

.EQU REG_TM0CNT, 0x4000100

.EQU REG_DMA1SAD, 0x40000BC
.EQU REG_DMA1DAD, 0x40000C0
.EQU REG_DMA1CNT, 0x40000C4

.EQU REG_DMA2SAD, 0x40000C8
.EQU REG_DMA2DAD, 0x40000CC
.EQU REG_DMA2CNT, 0x40000D0

.EQU REG_DMA3SAD, 0x40000D4
.EQU REG_DMA3DAD, 0x40000D8
.EQU REG_DMA3CNT, 0x40000DC

.EQU REG_SGFIFOA, 0x40000A0
.EQU REG_SGFIFOB, 0x40000A4
.EQU REG_VCOUNT, 0x4000006

@ MIXER CHANNEL FORMAT

.equ CHN_SIZE, 24

.equ CHN_SRC,0
.equ CHN_READ,4
.equ CHN_VOL,8
.equ CHN_PAN,9


.equ CHN_FREQ,12
.equ CHN_SIZE,16




@-------------------------------------

.equ FETCH_SIZE, 384
.equ FETCH_THRESHOLD, (6016)

@======================================================================
@ MEMORY
@======================================================================

.section .bss
.align 2

mp_writepos: .space 4 @ wavebuffer write position

mm_mixbuffer: .space 4 @ dont move (see init)
mm_mixchannels: .space 4 @ dont move
mm_wavebuffer: .space 4 @ dont move
mm_mixlen: .space 4 @ dont move
mm_mixch_count: .space 4 @ dont move
mm_mixch_end: .space 4 @ dont move
mm_ratescale: .space 4 @ dont move
mm_timerfreq: .space 4 @ dont move

mm_bpmdv: .space 4

mp_mix_seg: .space 1 @ mixing segment select

.align 2

mm_fetch: .space FETCH_SIZE+16

@ 11-bit mixed sample buffer
@ data is interleaved
@ left,left,right,right,left,left,etc...

@===========================================================================
@ PROGRAM
@===========================================================================

.section .iwram, "ax", %progbits

.ARM
.ALIGN 2

mpm_nullsample:
.byte 128

.align 2
@-----------------------------------------------------------------------------------------------------
mmVBlank: @ vblank wrapper, used to reset dma...HIGH PRIORITY PLEASE!
@-----------------------------------------------------------------------------------------------------

 b .mpvb_disabled @ disable until ready.. (overwrite this area with NOP)

 ldr r0,=mp_mix_seg @ swap mixing segmentl
 ldrsb r1, [r0]
 mvns r1, r1
 strb r1, [r0]
 beq .mpvb_exit

 ldr r1,=0x040000c6 @ dma control RESTART DMA
 ldr r0,=0x0440 @ disable dma
 strh r0, [r1]
 strh r0, [r1,#12]
 ldr r0,=0xB600 @ restart dma
 strh r0, [r1]
 strh r0, [r1,#12]

 b .mpvb_dontreset
.mpvb_exit:
 ldr r0,=mp_writepos @ restart write position
@ ldr r1,=mp_playbuffer_l @ ...
 ldr r1,=mm_wavebuffer
 ldr r1, [r1]
 str r1, [r0] @ ...
.mpvb_dontreset:

.mpvb_disabled:
 ldr r0,=0x3007FF8 @ acknowledge interrupt
 ldrh r2, [r0]
 orr r2, r2, #0x1
 strh r2, [r0]

 ldr r0,=mm_vblank_function
 ldr r0, [r0]
 cmp r0, #0
 bxeq lr
 bx r0
.pool

.align 2
@-------------------------------------------------------------------------
mmMixerMix: @ params={ samples_count }
@-------------------------------------------------------------------------

@ exit function if samples == 0
@ it will malfunction.

 cmp r0, #0
 bne .mp_zerocheck
 bx lr

.mp_zerocheck:

@ preserve registers

 stmfd sp!, {r4-r11,lr}

 stmfd sp!, {r0} @ preserve mixing count

@------------------------------------------------------------------------
@ SECTOR 0, INITIALIZATION
@------------------------------------------------------------------------

@ clear mixing buffers

 and r10, r0, #7
 mov r2, r0, lsr#3 @ clearing samps*2*2 bytes (hword*stereo) 32 bytes at a time
 ldr r0,=mm_mixbuffer
 ldr r0,[r0]
 mov r1, #0 @ zero variable
 mov r3, r1
 mov r4, r1
 mov r5, r1
 mov r6, r1
 mov r7, r1
 mov r8, r1
 mov r9, r1

 cmp r2, #0
 beq 2f

@ clear 32 bytes/write

1: stmia r0!, {r1,r3-r9}
 subs r2, r2, #1
 bne 1b
2:

@ clear remainder
 cmp r10, #0
 beq 2f

1: str r1, [r0], #4
 subs r10, r10, #1
 bne 1b
2:

@----------------------------------------------------------------------------------
@ BEGIN MIXING ROUTINE
@----------------------------------------------------------------------------------

 ldr r12,=mm_mixchannels
 ldr r12,[r12]
 mov r11, #0 @ volume addition

@--------------------
.mpm_cloop:
@--------------------

@----------------------------------------------------------------------
@ SECTOR 1, CALCULATIONS
@----------------------------------------------------------------------

@ aliases
# 291 "mm_mixer_gba.S"
@ read source address

 ldr r10, [r12, #CHN_SRC]
 cmp r10, #0
 bmi .mpm_next @ EXIT if MSB is set ------------>

@ read frequency value

 ldr r9, [r12, #CHN_FREQ]
 cmp r9, #0
 beq .mpm_next @ EXIT if zero ----------------=->

 ldr r0,=mm_ratescale
 ldr r0, [r0]
 mul r9, r0
 lsr r9, #14

@ load mixing buffers

 ldr r8,=mm_mixbuffer
 ldr r8,[r8]

@ get read position

 ldr r7, [r12, #CHN_READ]

@ calculate volume

 ldrb r6,[r12, #CHN_VOL] @ volume = 0-255
 ldrb r0, [r12, #CHN_PAN] @ pan = 0-255

 rsb r0, r0, #256
 mul r5, r0, r6 @ (vol*pan) = right volume
 mov r5, r5, lsr#8
 add r11, r11, r5 @ add to volume counter
 rsb r0, r0, #256
 mul r6, r0, r6 @ calc left volume (256-pan)*vol
 mov r6, r6, lsr#8
 add r11, r11, r6, lsl#16

 ldr r4, [sp] @ get mix count

@****************************************************************
.mpm_remix_test:
@****************************************************************

 mov r2, #0
 mul r1, r4, r9

 cmp r9, #FETCH_THRESHOLD
 bge 1f

 cmp r1, #FETCH_SIZE<<12
 movhi r1, #FETCH_SIZE<<12
 movhi r2, #1

1:

 ldr r0, [r10, #-C_SAMPLE_DATA+C_SAMPLE_LEN]
 rsb r0, r7, r0, lsl#SAMPFRAC

 cmp r1, r0
 movhi r1, r0
 bhi .calc_mix
 cmp r2, #0
 beq .mpm_mix_full

.calc_mix:


 push {r1}



.macro DIV_ITER shift

 cmp r0, r9, lsl#\shift
 subcs r0, r9, lsl#\shift
 addcs r2, #1<<\shift
.endm

.macro DIVIDER shift

 DIV_ITER \shift
.if \shift != 0
 DIVIDER (\shift-1)
.endif
.endm


 mov r0, r1
 mov r2, #0

1: subs r0, r9, lsl#16
 addcs r2, #1<<16
 bcs 1b
 add r0, r9, lsl#16

 DIVIDER 15

 cmp r0, #1
 adc r0, r2, #0







 pop {r1}
 sub r4, r0
 mov r3, r0
 b .mpm_mix_short

@------------------------------------------------------------------------
@ SECTOR 2, MIXING
@------------------------------------------------------------------------

@-----------------------------------------
.mpm_mix_full:
@-----------------------------------------

@ mix all samples

 mov r3, r4 @ <-- move mixing count
 mov r4, #0 @ clear mixing count
.mpm_mix_short:

@------------------------------------------------------
.mpm_remix:
@------------------------------------------------------

@ mix samples...
@ preserve registers
 stmfd sp!, {r4,r11,r12}

@ zero mixing count??
 cmp r3, #0
 beq .mpm_mix_complete @ exit -------->

 cmp r9, #FETCH_THRESHOLD
 bge .dont_use_fetch
# 450 "mm_mixer_gba.S"
 push {r3-r12}

 ldr r0,=mm_fetch
 add r10, r10, r7, lsr#12
 bic r10, #0b11
 add r1, #4<<12
 subs r1, #40<<12

 bcc .exit_fetch
.fetch: ldmia r10!, {r2-r9,r11,r14}
 stmia r0!, {r2-r9,r11,r14}
 subs r1, #40<<12
 bcc .exit_fetch
 ldmia r10!, {r2-r9,r11,r14}
 stmia r0!, {r2-r9,r11,r14}
 subs r1, #40<<12
 bcc .exit_fetch
 ldmia r10!, {r2-r9,r11,r14}
 stmia r0!, {r2-r9,r11,r14}
 subs r1, #40<<12
 bcs .fetch

.exit_fetch:

 adds r1, #(40<<12)-(24<<12)
 bmi .end_medfetch
.medfetch:
 ldmia r10!, {r2-r7}
 stmia r0!, {r2-r7}
 subs r1, #24<<12
 bcc .end_medfetch
 ldmia r10!, {r2-r7}
 stmia r0!, {r2-r7}
 subs r1, #24<<12
 bcc .end_medfetch
 ldmia r10!, {r2-r7}
 stmia r0!, {r2-r7}
 subs r1, #24<<12
 bcs .medfetch
.end_medfetch:

 adds r1, #24<<12
 bmi .end_fetch
.fetchsmall:
 ldr r2, [r10], #4
 str r2, [r0], #4
 subs r1, #4<<12
 ble .end_fetch
 ldr r2, [r10], #4
 str r2, [r0], #4
 subs r1, #4<<12
 bgt .fetchsmall
.end_fetch:

 pop {r3-r12}

fooo:

 mov r0, r7, lsr#12
 push {r0, r10}
 bic r7, r0, lsl#12
 and r0, #0b11
 ldr r10,=mm_fetch
 add r10, r10, r0

.dont_use_fetch:

 tst r8, #0b11
 beq .mpm_aligned




       @ routine to WORD align mixing sector
 ldrb r0, [r10, r7, lsr#SAMPFRAC] @ load sample
 add r7, r7, r9 @ add frequency
 mul r2, r0, r5 @ multiply by left volume
 ldrh r1, [r8] @ add to mixing buffer (left)
 add r1, r1, r2, lsr#5
 strh r1, [r8], #4
 mul r2, r0, r6 @ multiply by right volume
 ldrh r1, [r8] @ add to mixing buffer (right)
 add r1, r1, r2, lsr#5
 strh r1, [r8], #2
 sub r3, r3, #1 @ decrement mix count





.mpm_aligned:

 cmp r5, r6
 beq .mpm_mix_ac
 cmp r5, #0
 beq .mpm_mix_ar
 cmp r6, #0
 beq .mpm_mix_al
 b mmMix_ArbPanning
.mpm_mix_al:
 b mmMix_HardLeft
.mpm_mix_ar:
 b mmMix_HardRight

@ center mixing------------
.mpm_mix_ac:
 cmp r5, #0
 bne mmMix_CenteredPanning
 b mmMix_Skip
.mpm_mix_complete:

 cmp r9, #FETCH_THRESHOLD
 poplt {r0, r10}
 addlt r7, r7, r0, lsl#12
 ldmfd sp!, {r4,r11,r12}

 ldr r1, [r10, #-C_SAMPLE_DATA+C_SAMPLE_LEN]
 lsl r1, #SAMPFRAC

 cmp r1, r7
 bgt .mpm_channelfinished



 ldr r1, [r10, #-C_SAMPLE_DATA+C_SAMPLE_LOOP]
 cmp r1, #0

 bmi .mpm_channel_stop


 sub r7,r7,r1,lsl#(SAMPFRAC)

 cmp r4, #0
 ble .mpm_channelfinished
 b .mpm_remix_test

@----------------------------------------------------------------
.mpm_channel_stop:
@----------------------------------------------------------------

@ *** END OF SAMPLE
 mov r1, #1<<31
 str r1, [r12, #CHN_SRC]

@ mix zero into the rest of the buffer
 ldr r10,=mpm_nullsample
 mov r9, #0
 mov r7, #0
 movs r3, r4
 ble .mpm_channelfinished
 mov r4, #0
 mov r1, #0
 b .mpm_remix

@---------------------------------------------------------------
.mpm_channelfinished:
@---------------------------------------------------------------

 cmp r4, #0
 bne .mpm_remix_test

@ *** END OF MIXING ***

 str r7, [r12, #CHN_READ]

@-----------------------
.mpm_next:
@-----------------------

 add r12, r12, #CHN_SIZE
 ldr r0,=mm_mixch_end
 ldr r0,[r0]
 cmp r12, r0
 bne .mpm_cloop

@----------------------------------------------------------------------------------
@ SECTOR 3, POST-PROCESSING
@----------------------------------------------------------------------------------
# 639 "mm_mixer_gba.S"
 ldr r0,=mm_mixbuffer
 ldr r0,[r0]

 ldr r2,=mp_writepos
 ldr r2, [r2]
 ldr r3,=mm_mixlen
 ldr r3, [r3]
 add r3, r2, r3, lsl#1 @#MP_MIXLEN*2
 ldmfd sp!, {r4}

@ get volume accumulators

 mov r12, r11, lsr#16+1
 mov r12, r12, lsl#3

 mov r11, r11, lsl#16
 mov r11, r11, lsr#16+1
 mov r11, r11, lsl#3

 subs r4, r4, #1
 ble .mpm_copy2_end

@--------------------------------------------------
.mpm_copy2:
@--------------------------------------------------

@ ***************** LEFT OUTPUT ******************

 ldr r6, [r0], #4 @ get 2 mixed samples
 sub r5, r6, r11 @ convert to signed

 mov r5, r5, lsl#16 @ mask low hword with sign extension
 movs r5, r5, asr#16+3 @ and convert 11-bit to 8-bit

 cmp r5, #-128 @ clamp
 movlt r5, #-128 @
 cmp r5, #127 @
 movgt r5, #127 @

      @ next sample...
 rsbs r7, r11, r6,lsr#16 @ convert to signed
 movs r7, r7, asr#3 @ convert 11-bit to 8-bit

 cmp r7, #-128 @ clamp
 movlt r7, #-128 @
 cmp r7, #127 @
 movgt r7, #127 @

 and r5, r5, #255 @ write to output
 orr r5, r7, lsl#8 @
 strh r5, [r2], #2 @

@ **************** RIGHT OUTPUT ******************

 ldr r6, [r0], #4 @ get 2 mixed samples
 sub r5, r6, r12 @ convert to signed

 mov r5, r5, lsl#16 @ mask low hword and convert 11-bit to 8-bit
 movs r5, r5, asr#16+3 @

 cmp r5, #-128 @ clamp value
 movlt r5, #-128 @
 cmp r5, #127 @
 movgt r5, #127 @

      @ next sample...
 rsbs r7, r12, r6,lsr#16 @ convert to signed
 movs r7, r7, asr#3 @ convert 11-bit to 8-bit

 cmp r7, #-128 @ clamp value
 movlt r7, #-128 @
 cmp r7, #127 @
 movgt r7, #127 @

 and r5, r5, #255 @ write to output
 orr r5, r7, lsl#8 @
 strh r5, [r3], #2 @

 subs r4, r4, #2 @ loop
 bgt .mpm_copy2 @

@------------------------------------------------------------------

.mpm_copy2_end:
 ldr r0,=mp_writepos @ store new write position
 str r2, [r0]

@------------------------------------------------------------------

 ldmfd sp!, {r4-r11,lr}
 bx lr

.pool

@================================================================================
@ MIXING ROUTINES
@================================================================================

.macro READ_AND_INCREMENT reg
 ldrb \reg, [r10, r7, lsr#SAMPFRAC]
 add r7, r7, r9
.endm

.macro READ_D reg, tmp
 READ_AND_INCREMENT \reg
 READ_AND_INCREMENT \tmp
 orr \reg, \tmp, lsl#16
.endm

.macro MIX_D vol, tmp1, tmp2
 READ_D \tmp1, \tmp2
 mul \tmp2, \tmp1, \vol
 bic \tmp2, \tmp2, #0x1F0000
.endm

.macro MIX_DA vol, target, tmp1, tmp2
 MIX_D \vol, \tmp1, \tmp2
 add \target, \target, \tmp2, lsr#5
.endm

.macro MIX_DC vol, target_a, target_b, tmp1, tmp2
 MIX_D \vol, \tmp1, \tmp2
 add \target_a, \target_a, \tmp2, lsr#5
 add \target_b, \target_b, \tmp2, lsr#5
.endm

.macro MIX_DB vol_l, vol_r, target_a, target_b, tmp1, tmp2
 READ_D \tmp1, \tmp2

.if \target_a != 0
 mul \tmp2, \tmp1, \vol_l
 bic \tmp2, \tmp2, #0x1F0000
 add \target_a, \target_a, \tmp2, lsr#5
.endif
.if \target_b != 0
 mul \tmp2, \tmp1, \vol_r
 bic \tmp2, \tmp2, #0x1F0000
 add \target_b, \target_b, \tmp2, lsr#5
.endif
.endm

@-----------------------------------------------------------------------------------
mmMix_Skip:
@-----------------------------------------------------------------------------------

@ mix nothing
 mul r0, r3, r9 @ read += samples * frequency
 add r7, r7, r0
 b .mpm_mix_complete

@-----------------------------------------------------------------------------------
mmMix_HardLeft:
@-----------------------------------------------------------------------------------
@ mix hard panned left

 bl mmMix_SingleChannel
 bgt mmMix_Remainder
 b .mpm_mix_complete

@-----------------------------------------------------------------------------------
mmMix_HardRight:
@-----------------------------------------------------------------------------------
@ hard panned right

 mov r5, r6
 add r8, r8, #4
 bl mmMix_SingleChannel
 mov r5, #0
 sub r8, r8, #4
 bgt mmMix_Remainder
 b .mpm_mix_complete

@----------------------------------------
mmMix_SingleChannel:
@----------------------------------------
# 822 "mm_mixer_gba.S"
@ hard panned mixing (single channel mono)
@ interleaving really cuts this method's effectiveness :(

@ mix 8 samples/loop

 subs r3, r3, #8
 bmi .mpmah_8e
.mpmah_8:
 ldmia r8, {r1, r2, r11}
 MIX_DA r5, r1, r0, r4
 str r1, [r8], #8
 MIX_DA r5, r11, r0, r4
 str r11, [r8], #8
 ldmia r8, {r1, r2, r11}
 MIX_DA r5, r1, r0, r4
 str r1, [r8], #8
 MIX_DA r5, r11, r0, r4
 str r11, [r8], #8
 subs r3, r3, #8
 bpl .mpmah_8
.mpmah_8e:

@ mix remainder samples

 adds r3, r3, #8
 bx lr
# 856 "mm_mixer_gba.S"
@----------------------------------------------------------
mmMix_CenteredPanning:
@----------------------------------------------------------
# 869 "mm_mixer_gba.S"
@ mix center panning (double channel mono)

 subs r3, r3, #6
 bmi .mpmac_6e
.mpmac_6:

 ldmia r8, {r1,r2,r4,r6,r11,r12}
 MIX_DC r5, r1, r2, r0, lr
 MIX_DC r5, r4, r6, r0, lr
 MIX_DC r5, r11, r12, r0, lr
 stmia r8!, {r1,r2,r4,r6,r11,r12}
 subs r3, r3, #6
 bpl .mpmac_6
.mpmac_6e:

 mov r6, r5
 adds r3, r3, #6
 bgt mmMix_Remainder
 b .mpm_mix_complete
# 898 "mm_mixer_gba.S"
@---------------------------------------------------
mmMix_ArbPanning:
@---------------------------------------------------
# 910 "mm_mixer_gba.S"
 subs r3, r3, #10
 bmi .mpmaa_10e
.mpmaa_10:

 ldmia r8, {r1,r2,r4,r11,r14}
 MIX_DB r5, r6, r1, r2, r0, r12
 MIX_DB r5, r6, r4, r11, r0, r12
 MIX_DB r5, r6, r14, 0, r0, r12
 stmia r8!, {r1,r2,r4,r11,r14}
 ldmia r8, {r1,r2,r4,r11,r14}
 mul r12, r0, r6
 bic r12, r12, #0x1F0000
 add r1, r1, r12, lsr#5
 MIX_DB r5, r6, r2, r4, r0, r12
 MIX_DB r5, r6, r11, r14, r0, r12
 stmia r8!, {r1,r2,r4,r11,r14}
 subs r3, r3, #10
 bpl .mpmaa_10
.mpmaa_10e:

 adds r3, r3, #10
 bgt mmMix_Remainder
 b .mpm_mix_complete

.pool
# 944 "mm_mixer_gba.S"
mmMix_Remainder:
# 955 "mm_mixer_gba.S"
 orr r11, r5, r6, lsl#16

.mix_remaining:
# 970 "mm_mixer_gba.S"
 ldrb r0, [r10, r7, lsr#SAMPFRAC] @ 3 load sample
 add r7, r7, r9 @ 1 add frequency
 mul r4, r11, r0 @ 2 multiply by volume
 ldrh r1, [r8] @ 3 load mix buffer entry (left)
 bic r2, r4, #0xFF0000 @ 1 prep for shift
 add r1, r1, r2, lsr#5 @ 1 add
 strh r1, [r8], #2 @ 2 store (left)
 ldrh r1, [r8, #2] @ 3 load (right)
 add r1, r1, r4, lsr#16+5 @ 1 add values
 strh r1, [r8, #2] @ 2 store (right)

 subs r3, r3, #2 @ 2
 blt .end_mix_remaining @ 1/exit
# 996 "mm_mixer_gba.S"
 ldrb r0, [r10, r7, lsr#SAMPFRAC] @ 3 load sample
 add r7, r7, r9 @ 1 add frequency
 mul r4, r11, r0 @ 2 multiply by volume
 ldrh r1, [r8] @ 3 load mix buffer entry (left)
 bic r2, r4, #0xFF0000 @ 1 prep for shift
 add r1, r1, r2, lsr#5 @ 1 add
 strh r1, [r8], #4 @ 2 store (left)
 ldrh r1, [r8] @ 3 load (right)
 add r1, r1, r4, lsr#16+5 @ 1 add values
 strh r1, [r8], #2 @ 2 store (right)

 bgt .mix_remaining
.end_mix_remaining:

 b .mpm_mix_complete







@============================================================================
@ END OF MIXER
@============================================================================

 .TEXT
 .THUMB
 .ALIGN 2






     .thumb_func
mmMixerSetSource:

 mov r2, #CHN_SIZE
 mul r0, r2
 ldr r2,=mm_mixchannels
 ldr r2, [r2]
 add r0, r2

 add r1, #C_SAMPLE_DATA
 str r1, [r0, #CHN_SRC]

 mov r1, #0
 str r1, [r0, #CHN_READ]

 bx lr






     .thumb_func
mmMixerSetRead:

 mov r2, #CHN_SIZE
 mul r0, r2
 ldr r2,=mm_mixchannels
 ldr r2,[r2]
 add r0, r2

 str r1, [r0, #CHN_READ]
 bx lr






     .thumb_func
mmMixerSetFreq:



 mov r2, #CHN_SIZE
 mul r0, r2, r0
 ldr r2,=mm_mixchannels
 ldr r2,[r2]
 add r0, r0, r2

 lsl r1, #2

 strh r1, [r0, #CHN_FREQ]
 bx lr
# 1111 "mm_mixer_gba.S"
 .THUMB






     .thumb_func
mmMixerMulFreq:

 mov r2, #CHN_SIZE
 mul r0, r2
 ldr r2,=mm_mixchannels
 ldr r2, [r2]
 add r0, r2

 ldr r3, [r0, #CHN_FREQ]
 mul r3, r1
 lsr r3, #10
 str r3, [r0, #CHN_FREQ]
 bx lr






     .thumb_func
mmMixerStopChannel:

 mov r1, #CHN_SIZE
 mul r0, r1
 ldr r1,=mm_mixchannels
 ldr r1,[r1]
 add r0, r1

 mov r1, #1
 lsl r1, #31
 str r1, [r0]
 bx lr






     .thumb_func
mmMixerChannelActive:

 mov r1, #CHN_SIZE
 mul r0, r1
 ldr r1,=mm_mixchannels
 ldr r1,[r1]
 add r0, r1

mp_Mixer_ChannelEnabledA:
 ldr r0, [r0, #CHN_SRC]
 asr r0, #31
 mvn r0, r0
 bx lr






     .thumb_func
mmMixerSetVolume:

 mov r2, #CHN_SIZE
 mul r0, r2
 ldr r2,=mm_mixchannels
 ldr r2,[r2]
 add r0, r2

 strb r1, [r0, #CHN_VOL]

 bx lr






     .thumb_func
mmMixerSetPan:

 mov r2, #CHN_SIZE
 mul r0, r2
 ldr r2,=mm_mixchannels
 ldr r2,[r2]
 add r0, r2

 strb r1, [r0, #CHN_PAN]
 bx lr






     .thumb_func
mmMixerInit:

 ldr r2,=mm_mixbuffer
 ldr r1, [r0,#MM_GBA_SYSTEM_ACH_COUNT]
 str r1, [r2,#16]

 mov r3, #CHN_SIZE
 mul r1, r3
 ldr r3, [r0,#MM_GBA_SYSTEM_MIXCH]
 str r3, [r2,#4]
 add r3, r1
 str r3, [r2,#20]
 ldr r1, [r0,#MM_GBA_SYSTEM_MIXMEM]
 str r1, [r2,#0]
 ldr r1, [r0,#MM_GBA_SYSTEM_WAVEMEM]
 str r1, [r2,#8]
 ldr r1, [r0,#MM_GBA_SYSTEM_MODE]
 lsl r1, #1
 adr r3, mp_mixing_lengths
 ldrh r3, [r3,r1]
 str r3, [r2,#12]
 adr r3, mp_rate_scales
 ldrh r3, [r3, r1]
 str r3, [r2, #24]
 adr r3, mp_timing_sheet
 ldrh r3, [r3, r1]
 str r3, [r2, #28]
 adr r3, mp_bpm_divisors
 lsl r1, #1
 ldr r3, [r3,r1]

 ldr r2,=mm_bpmdv
 str r3, [r2,#0]


 ldr r0,=mm_wavebuffer @ clear wave buffer
 ldr r0,[r0]
 ldr r1,=mm_mixlen
 ldr r1, [r1]
 mov r2, #0 @ ..
.mpi_loop1: @ ..
 stmia r0!, {r2} @ ..
 sub r1, r1, #1 @ ..
 bne .mpi_loop1 @ ..

 ldr r0,=mp_mix_seg @ reset mixing segment
 strb r2, [r0] @ ..

 ldr r0,=mm_mixchannels @ disable mixing channels
 ldr r1,[r0,#12]@ nchannels
 ldr r0,[r0]
 ldr r3,=1<<31
.mpi_loop2:
 str r3, [r0, #CHN_SRC]
 add r0, #CHN_SIZE
 sub r1, #1
 bne .mpi_loop2

 ldr r0,=mmVBlank @ enable vblank routine
 ldr r1,=0xE1A00000 @ ..
 str r1, [r0] @ ..

 ldr r0,=REG_SGFIFOA @ clear fifo data
 str r2, [r0] @ ..
 str r2, [r0, #4] @ ..
 ldr r0,=REG_SOUNDCNT_H @ reset direct sound
 strh r2, [r0] @ ..
 ldr r1,=0x9A0C @ setup sound [DIRECT SOUND A/B reset,timer0,A=left,B=right,volume=100%]
 strh r1, [r0] @ ..
 ldr r0,=REG_DMA1SAD @ setup DMA source addresses (playback buffers)
 ldr r1,=mm_wavebuffer
 ldr r2, [r1, #4]@mixlen
 ldr r1, [r1]

 @ldr r1,=mp_playbuffer_l @ ..
 str r1, [r0] @ ..

 add r1,r2
 add r1,r2

@ ldr r1,=mp_playbuffer_r @ ..
 str r1, [r0, #12] @ ..

 ldr r1,=REG_SGFIFOA @ setup DMA destination (sound fifo)
 str r1, [r0, #4] @ ..
 add r1, #4 @ ..
 str r1, [r0, #16] @ ..

 ldr r1,=0xB6000000 @ enable DMA (enable,fifo request,32-bit,repeat)
 str r1, [r0, #8] @ ..
 str r1, [r0, #20] @ ..

 ldr r0,=REG_SOUNDCNT_X @ master sound enable
 mov r1, #0x80 @ ..
 strh r1, [r0] @ ..

 ldr r0,=REG_VCOUNT @ wait for new frame
.mpi_vsync: @ ..
 ldrh r1, [r0] @ skip current vblank period
 cmp r1, #160 @ ..
 bge .mpi_vsync @ ..
.mpi_vsync2:
 ldrh r1, [r0] @ wait for new one
 cmp r1, #160 @ ..
 blt .mpi_vsync2 @ ..

.mpi_vsync_2: @ pass#2
 ldrh r1, [r0] @ skip current vblank period
 cmp r1, #160 @ ..
 bge .mpi_vsync_2 @ ..
.mpi_vsync2_2:
 ldrh r1, [r0] @ wait for new one
 cmp r1, #160 @ ..
 blt .mpi_vsync2_2 @ ..

 ldr r0,=REG_TM0CNT @ enable sampling timer
 ldr r1,=mm_timerfreq
 ldr r1,[r1]
 mov r2, #0x80
 lsl r2, #16
 orr r1, r2
 @ldr r1,=(-MP_TIMERFREQ&0xFFFF) | (0x80<<16) @ ..
 str r1, [r0] @ ..
 bx lr @ finished



.align 2
mp_mixing_lengths:
 .hword 136, 176, 224, 264, 304, 352, 448, 528
 @ 8khz,10khz,13khz,16khz,18khz,21khz,27khz,32khz

.align 2




mp_rate_scales:
 .hword 31812, 24576, 19310, 16384, 14228, 12288, 9655, 8192
 @ 8khz, 10khz, 13khz, 16khz, 18khz, 21khz, 27khz, 32khz
 @ 8121, 10512, 13379, 15768, 18157, 21024, 26758, 31536,

.align 2

mp_timing_sheet:
 .hword -2066,-1596,-1254,-1064, -924, -798, -627, -532
 @ 8khz,10khz,13khz,16khz,18khz,21khz,27khz,32khz

.align 2

mp_bpm_divisors:
 .word 20302,26280,33447,39420,45393,52560,66895,78840


.end
