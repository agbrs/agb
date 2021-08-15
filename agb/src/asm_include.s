.macro agb_arm_func functionName:req
.section .iwram, "ax", %progbits @ "ax" = allocatable and executable, %progbits = contains data
.arm
.align 2
.global \functionName
.type \functionName, %function
.func \functionName
\functionName:
.endm

.macro agb_arm_end functionName:req
.pool
.size \functionName,.-\functionName
.endfunc
.endm
