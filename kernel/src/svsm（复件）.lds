ENTRY(_start)
SECTIONS
{
    . = 0x40080000;
    kernel_region_phys_start = .;
    kernel_text_start = .;
    .text :
    {
      KEEP(*(.text.boot))
      *(.text.exceptions)
      . = ALIGN(4096); /* align for exceptions_vector_table*/
      *(.text.exceptions_vector_table)
      *(.text)
    }
    kernel_text_end = .;
    kernel_data_start = .;
    .data : { 
      *(.data) 
    }
    kernel_data_end = .;
    kernel_rodata_start = .;
    .rodata : { 
      *(.rodata)
    }
    kernel_rodata_end = .;
    kernel_bss_start = .;
    .bss : { 
      *(.bss) 
    }
    kernel_bss_end = .;

    . = ALIGN(4096);
    LD_TTBR0_BASE = .;
    . = . + 0x1000;

    LD_TTBR1_BASE = .;
    . = . + 0x1000;

    . = ALIGN(8);
    . = . + 0xF0000;
    LD_STACK_PTR = .;

    . = . + 0xF0000;
    kernel_region_phys_end = .;
}