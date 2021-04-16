OUTPUT_FORMAT("elf32-littlearm", "elf32-bigarm", "elf32-littlearm")
OUTPUT_ARCH(arm)

ENTRY(__start)

MEMORY {
    ewram (w!x) : ORIGIN = 0x02000000, LENGTH = 256K
    iwram (w!x) : ORIGIN = 0x03000000, LENGTH = 32K
    rom (rx)    : ORIGIN = 0x08000000, LENGTH = 32M
}

__text_start = ORIGIN(rom);

SECTIONS {
    . = __text_start;

    .crt0 : {
		KEEP (crt0.o(.text));
		. = ALIGN(4);
    } > rom

    .text : {
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

        *(.data .data.*);
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

        __ewram_data_end = ABSOLUTE(.);
    } > ewram AT>rom
    
    .bss : {
        *(.bss .bss.*);
        . = ALIGN(4);
    } > iwram

    __iwram_rom_length_bytes = __iwram_data_end - __iwram_data_start;
    __iwram_rom_length_halfwords = (__iwram_rom_length_bytes + 1) / 2;

    __ewram_rom_length_bytes = __ewram_data_end - __ewram_data_start;
    __ewram_rom_length_halfwords = (__ewram_rom_length_bytes + 1) / 2;

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}