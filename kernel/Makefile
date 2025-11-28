KERNEL := target/build/aarch64-unknown-none-softfloat/debug/svsm
QEMU := qemu-system-aarch64
TARGET = aarch64-unknown-none-softfloat

QEMU_ARGS := \
	-machine virt,gic-version=3 \
	-cpu cortex-a53 \
	-nographic \
	-smp 1 \
	-m 1024M \
	-kernel $(KERNEL) 

QEMU_ARGS_DEBUG := \
	-machine virt,gic-version=3 \
	-cpu cortex-a53 \
	-nographic \
	-smp 1 \
	-m 1024M \
	-kernel $(KERNEL) \
	-S -s

.PHONY: run
run:
	@echo "Launching QEMU with kernel: $(KERNEL)"
	$(QEMU) $(QEMU_ARGS)

.PHONY: kill
kill:
	@echo "Killing all QEMU instances..."
	@pkill -9 qemu-system-aarch64 || true

.PHONY: show
show:
	@echo "$(QEMU) $(QEMU_ARGS)"

.PHONY: build
build:
	cargo build --target $(TARGET) --target-dir target/build
	
.PHONY: tpm
tpm:
	cargo build --target $(TARGET) --features vtpm --target-dir target/vtpm

.PHONY: cca
cca:
	cargo build --target $(TARGET) --features cca --target-dir target/cca
	
.PHONY: debug
debug:
	@echo "Debug QEMU with kernel: $(KERNEL)"
	$(QEMU) $(QEMU_ARGS_DEBUG)

.PHONY: gdb
gdb:
	gdb-multiarch target/build/aarch64-unknown-none-softfloat/debug/svsm
	
.PHONY: build-release
build-release:
	cargo build --target aarch64-unknown-none-softfloat --release
	
