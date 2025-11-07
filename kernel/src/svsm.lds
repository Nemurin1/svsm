ENTRY(_start)
SECTIONS
{
    . = 0x40080000;
    kernel_region_phys_start = .;
    .text.boot : { *(.text.boot) }
    .text :
    {
      KEEP(*(.text.boot))
      *(.text.exceptions)
      . = ALIGN(4096); /* align for exceptions_vector_table*/
      *(.text.exceptions_vector_table)
      *(.text)
    }
    .data : { 
      *(.data) 
    }
    .rodata : { 
      *(.rodata)
    }
    .bss : { 
      *(.bss) 
    }

    . = ALIGN(4096);
    LD_TTBR0_BASE = .;
    . = . + 0x1000;

    LD_TTBR1_BASE = .;
    . = . + 0x1000;

    . = ALIGN(8);
    . = . + 0xF000;
    LD_STACK_PTR = .;

    . = . + 0x1000;
    kernel_region_phys_end = .;
}