OUTPUT_FORMAT("elf32-littlearm", "elf32-bigarm", "elf32-littlearm")
OUTPUT_ARCH(arm)

ENTRY(__start)

MEMORY {
    ewram (w!x) : ORIGIN = 0x2000000, LENGTH = 256K
    iwram (w!x) : ORIGIN = 0x3000000, LENGTH = 32K
    rom (rx)    : ORIGIN = 0x8000000, LENGTH = 32M
}

__text_start = ORIGIN(rom);

SECTIONS {
    . = __text_start;

    .crt0 : {
		KEEP (*(.crt0));
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

    __data_start = .;
    .data : {
        *(.data .data.*);
        . = ALIGN(4);
    } > iwram
    __data_end = .;

    .bss : {
        *(.bss .bss.*);
        . = ALIGN(4);
    } > iwram

    .stack 0x80000 : {
        *(.stack)
    }

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}