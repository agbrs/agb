.macro agb_arm_func functionName:req
.section .iwram.\functionName, "ax", %progbits
.arm
.align 2
.global \functionName
.type \functionName, %function
@ .func \functionName
\functionName:
.endm

.macro agb_arm_end functionName:req
.pool
.size \functionName,.-\functionName
@ .endfunc
.endm

.macro agb_thumb_func functionName:req
.section .iwram.\functionName, "ax", %progbits
.thumb
.align 1
.global \functionName
.type \functionName, %function
@ .func \functionName
\functionName:
.endm

.macro agb_thumb_end functionName:req
.pool
.size \functionName,.-\functionName
@ .endfunc
.endm
