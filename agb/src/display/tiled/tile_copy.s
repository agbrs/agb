agb_arm_func copy_tile_8bpp
    @ Arguments
    @ r0 - pointer to the image data beginning
    @ r1 - pointer to the target in vram
    push {{r4-r8}}

.rept 2
    ldmia r0!, {{r2-r8, r12}}
    stmia r1!, {{r2-r8, r12}}
.endr

    pop {{r4-r8}}
    bx lr
agb_arm_end copy_tile_8bpp

agb_arm_func copy_tile_4bpp
    @ Arguments
    @ r0 - pointer to the image data beginning
    @ r1 - pointer to the target in vram
    push {{r4-r8}}

    ldmia r0!, {{r2-r8, r12}}
    stmia r1!, {{r2-r8, r12}}

    pop {{r4-r8}}
    bx lr
agb_arm_end copy_tile_4bpp