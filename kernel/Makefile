# ELF 可执行文件路径（父目录中的 target）
KERNEL := ../target/aarch64-unknown-none-softfloat/debug/svsm

# QEMU 可执行文件
QEMU := qemu-system-aarch64

# QEMU 运行参数
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

# 可选：如果你需要用到 GDB 调试，可以这样添加：
# QEMU_ARGS += -s -S

# 默认目标
.PHONY: run
run:
	@echo "Launching QEMU with kernel: $(KERNEL)"
	$(QEMU) $(QEMU_ARGS)

# 清理 QEMU 进程（可选）
.PHONY: kill
kill:
	@echo "Killing all QEMU instances..."
	@pkill -9 qemu-system-aarch64 || true

# 显示完整命令（调试用）
.PHONY: show
show:
	@echo "$(QEMU) $(QEMU_ARGS)"

.PHONY: build
build:
	cargo build --target aarch64-unknown-none-softfloat 
	
.PHONY: tpm
tpm:
	cargo build --target aarch64-unknown-none-softfloat --features vtpm

.PHONY: debug
debug:
	@echo "Debug QEMU with kernel: $(KERNEL)"
	$(QEMU) $(QEMU_ARGS_DEBUG)

.PHONY: gdb
gdb:
	gdb-multiarch ../target/aarch64-unknown-none-softfloat/debug/svsm
	
.PHONY: build-release
build-release:
	cargo build --target aarch64-unknown-none-softfloat --release
	
