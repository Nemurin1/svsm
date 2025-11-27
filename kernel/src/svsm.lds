ENTRY(_start)
SECTIONS
{
    . = 0x40080000;
    kernel_region_phys_start = .;
    . = ALIGN(0x1000);
    .text :
    {
      KEEP(*(.text.boot))
      *(.text.exceptions)
      . = ALIGN(4096); /* align for exceptions_vector_table*/
      *(.text.exceptions_vector_table)
      *(.text*)
    }
    . = ALIGN(0x1000);
    .data : { 
      *(.data*) 
    }
    . = ALIGN(0x1000);
    .rodata : { 
      *(.rodata*)
    }
    . = ALIGN(0x1000);
    .bss : { 
      *(.bss*) 
    }

    . = ALIGN(0x1000);
    LD_TTBR0_BASE = .;
    . = . + 0x1000;

    LD_TTBR1_BASE = .;
    . = . + 0x1000;

    . = ALIGN(0x1000);
    . = . + 0x8000;
    LD_STACK_PTR = .;

    kernel_region_phys_end = .;
}