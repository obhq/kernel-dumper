SECTIONS {
    .text 0x00 : {
        *(.text.entry)
    }

    .dynamic 0x80 : {
        *(.dynamic)
    }

    .text : {
        *(.text.*)
    }

    .rodata : {
        *(.rela.dyn)
    }

    .bss : {
        *(.bss.*)
    }

    /DISCARD/ : {
        *(.dynsym)
        *(.gnu.hash)
        *(.hash)
        *(.dynstr)
    }
}
