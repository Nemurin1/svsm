CC      = aarch64-none-elf-gcc
AS      = aarch64-none-elf-as
AR      = aarch64-none-elf-ar

CFLAGS  = -O2 -ffreestanding -nostdlib -nostdinc -Wall -Wextra
ASFLAGS =
LIBNAME = libmylib.a

CFLAGS += -I./
ASFLAGS += -I./

CSRCS   = $(wildcard *.c)
ASRCS   = $(wildcard *.S)
OBJS    = $(CSRCS:.c=.o) $(ASRCS:.S=.o)

all: $(LIBNAME)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

%.o: %.s
	$(AS) $(ASFLAGS) $< -o $@

$(LIBNAME): $(OBJS)
	$(AR) rcs $(LIBNAME) $(OBJS)

clean:
	rm -f *.o $(LIBNAME)

