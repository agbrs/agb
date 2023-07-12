OUTPUT_FORMAT("elf32-littlearm", "elf32-bigarm", "elf32-littlearm")
OUTPUT_ARCH(arm)

ENTRY(__start)
EXTERN(__RUST_INTERRUPT_HANDLER)

EXTERN(__agbabi_memset)
EXTERN(__agbabi_memcpy)

MEMORY {
    ewram (w!x) : ORIGIN = 0x02000000, LENGTH = 256K
    iwram (w!x) : ORIGIN = 0x03000000, LENGTH = 32K
    rom (rx)    : ORIGIN = 0x08000000, LENGTH = 32M
}

__text_start = ORIGIN(rom);

SECTIONS {
    . = __text_start;


    .text : {
        KEEP(*(.crt0));
        *(.crt0 .crt0*);
        *(.text .text*);
        . = ALIGN(4);
    } > rom
    __text_end = .;

    .rodata : {
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } > rom

    __iwram_rom_start = .;
    .iwram : {
        __iwram_data_start = ABSOLUTE(.);

        *(.iwram .iwram.*);
        . = ALIGN(4);

        *(.text_iwram .text_iwram.*);
        . = ALIGN(4);

        __iwram_data_end = ABSOLUTE(.);
    } > iwram AT>rom

    . = __iwram_rom_start + (__iwram_data_end - __iwram_data_start);

    __ewram_rom_start = .;
    .ewram : {
        __ewram_data_start = ABSOLUTE(.);

        *(.ewram .ewram.*);
        . = ALIGN(4);

        *(.data .data.*);
        . = ALIGN(4);

        __ewram_data_end = ABSOLUTE(.);
    } > ewram AT>rom
    
    .bss : {
        *(.bss .bss.*);
        . = ALIGN(4);
        __iwram_end = ABSOLUTE(.);
    } > iwram

    __iwram_rom_length_bytes = __iwram_data_end - __iwram_data_start;
    __iwram_rom_length_halfwords = (__iwram_rom_length_bytes + 1) / 2;

    __ewram_rom_length_bytes = __ewram_data_end - __ewram_data_start;
    __ewram_rom_length_halfwords = (__ewram_rom_length_bytes + 1) / 2;

    .shstrtab : {
       *(.shstrtab)
    }

        /* debugging sections */
    /* Stabs */
    .stab            0 : { *(.stab) }
    .stabstr         0 : { *(.stabstr) }
    .stab.excl       0 : { *(.stab.excl) }
    .stab.exclstr    0 : { *(.stab.exclstr) }
    .stab.index      0 : { *(.stab.index) }
    .stab.indexstr   0 : { *(.stab.indexstr) }
    .comment         0 : { *(.comment) }
    /* DWARF 1 */
    .debug           0 : { *(.debug) }
    .line            0 : { *(.line) }
    /* GNU DWARF 1 extensions */
    .debug_srcinfo   0 : { *(.debug_srcinfo) }
    .debug_sfnames   0 : { *(.debug_sfnames) }
    /* DWARF 1.1 and DWARF 2 */
    .debug_aranges   0 : { *(.debug_aranges) }
    .debug_pubnames  0 : { *(.debug_pubnames) }
    /* DWARF 2 */
    .debug_info      0 : { *(.debug_info) }
    .debug_abbrev    0 : { *(.debug_abbrev) }
    .debug_line      0 : { *(.debug_line) }
   	.debug_frame     0 : { *(.debug_frame) }
    .debug_str       0 : { *(.debug_str) }
    .debug_loc       0 : { *(.debug_loc) }
    .debug_macinfo   0 : { *(.debug_macinfo) }
    /* SGI/MIPS DWARF 2 extensions */
    .debug_weaknames 0 : { *(.debug_weaknames) }
    .debug_funcnames 0 : { *(.debug_funcnames) }
    .debug_typenames 0 : { *(.debug_typenames) }
    .debug_varnames  0 : { *(.debug_varnames) }

    .debug_ranges   0 : { *(.debug_ranges) }

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}