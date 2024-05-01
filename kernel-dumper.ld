SECTIONS {
    .text 0x00 : {
        *(.text.entry)
    }

    .dynamic 0x100 : {
        *(.dynamic)
    }

    .rodata : ALIGN(8) {
        *(.rela.dyn)
    }

    .text : {
        *(.text.*)
    }

    /DISCARD/ : {
        *(.dynsym)
        *(.gnu.hash)
        *(.hash)
        *(.dynstr)
    }
}
