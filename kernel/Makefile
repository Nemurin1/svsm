KERNEL := target/build/aarch64-unknown-none-softfloat/debug/svsm
QEMU := qemu-system-aarch64
TARGET = aarch64-unknown-none-softfloat

QEMU_ARGS := \
	-machine virt,gic-version=3 \
	-cpu max \
	-dtb base.dtb \
	-nographic \
	-smp 1 \
	-m 1024M \
	-kernel $(KERNEL) \
	-device loader,file=target/build/aarch64-unknown-none-softfloat/debug/Image,addr=0x60000000 \
	
QEMU_ARGS_DEBUG := \
	-machine virt,gic-version=3 \
	-cpu max \
	-nographic \
	-smp 1 \
	-m 1024M \
	-dtb base.dtb \
	-kernel $(KERNEL) \
	-device loader,file=target/build/aarch64-unknown-none-softfloat/debug/Image,addr=0x60000000 \
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
	# cp target/build/aarch64-unknown-none-softfloat/debug/svsm ../../../cca-workspace/image-guest
	
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
	
.PHONY: plane
plane:
	cd ../plane && make
	
