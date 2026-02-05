#!/bin/bash
# test-qemu.sh - 简单的 QEMU 启动测试

TG18_DIR="/home/chyyuu/thecodes/os-compare/tg-ch18"
KERNEL="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/tg-ch18"
FS_IMG="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/fs.img"

echo "启动 tg-ch18 内核..."
echo "Ctrl-A X 退出 QEMU"
echo

qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -bios none \
    -drive file="$FS_IMG",if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -kernel "$KERNEL" \
    -nographic
