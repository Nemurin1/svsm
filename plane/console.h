#define UART0 0x09000000

static inline void uart_putc(char c)
{
    *(volatile unsigned char *)UART0 = c;
}

void uart_puts(const char *s)
{
    while (*s)
        uart_putc(*s++);
}