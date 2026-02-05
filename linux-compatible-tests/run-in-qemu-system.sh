#!/bin/bash
# run-in-qemu-system.sh - Launch tg-ch18 kernel with qemu-system-riscv64
#
# This script:
# 1. Verifies qemu-system-riscv64 is available
# 2. Launches the kernel with the filesystem image
# 3. Provides test options for running ch18_file* programs

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TG18_DIR="/home/chyyuu/thecodes/os-compare/tg-ch18"
KERNEL="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/tg-ch18"
FS_IMG="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/fs.img"

# Check dependencies
echo -e "${BLUE}=== Verifying dependencies ===${NC}"

if ! command -v qemu-system-riscv64 &> /dev/null; then
    echo -e "${RED}❌ qemu-system-riscv64 not found${NC}"
    echo "Install with: sudo apt install qemu-system-misc"
    exit 1
fi

if [ ! -f "$KERNEL" ]; then
    echo -e "${RED}❌ Kernel not found: $KERNEL${NC}"
    echo "Please run pack-to-fsimg.sh first"
    exit 1
fi

if [ ! -f "$FS_IMG" ]; then
    echo -e "${RED}❌ Filesystem image not found: $FS_IMG${NC}"
    echo "Please run pack-to-fsimg.sh first"
    exit 1
fi

echo -e "${GREEN}✓ qemu-system-riscv64 found$(qemu-system-riscv64 --version | head -1 | sed 's/^/: /')${NC}"
echo -e "${GREEN}✓ Kernel: $(ls -lh $KERNEL | awk '{print $5, $9}')${NC}"
echo -e "${GREEN}✓ Filesystem: $(ls -lh $FS_IMG | awk '{print $5, $9}')${NC}"
echo

# Determine test mode
TEST_MODE="${1:-interactive}"

case "$TEST_MODE" in
    interactive)
        echo -e "${BLUE}=== Launching tg-ch18 kernel (interactive mode) ===${NC}"
        echo "Type commands and press Ctrl-A X to exit"
        echo
        qemu-system-riscv64 \
            -machine virt \
            -m 64M \
            -bios none \
            -drive file="$FS_IMG",if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -kernel "$KERNEL" \
            -nographic
        ;;
    
    ch18_file0)
        echo -e "${BLUE}=== Testing ch18_file0 ===${NC}"
        echo "Running: ./ch18_file0"
        echo
        timeout 30 qemu-system-riscv64 \
            -machine virt \
            -m 64M \
            -bios none \
            -drive file="$FS_IMG",if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -kernel "$KERNEL" \
            -nographic 2>&1 || true
        ;;
    
    ch18_file1)
        echo -e "${BLUE}=== Testing ch18_file1 ===${NC}"
        echo "Running: ./ch18_file1"
        echo
        timeout 30 qemu-system-riscv64 \
            -machine virt \
            -m 64M \
            -bios none \
            -drive file="$FS_IMG",if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -kernel "$KERNEL" \
            -nographic 2>&1 || true
        ;;
    
    ch18_file2)
        echo -e "${BLUE}=== Testing ch18_file2 ===${NC}"
        echo "Running: ./ch18_file2"
        echo
        timeout 30 qemu-system-riscv64 \
            -machine virt \
            -m 64M \
            -bios none \
            -drive file="$FS_IMG",if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -kernel "$KERNEL" \
            -nographic 2>&1 || true
        ;;
    
    ch18_file3)
        echo -e "${BLUE}=== Testing ch18_file3 ===${NC}"
        echo "Running: ./ch18_file3"
        echo
        timeout 30 qemu-system-riscv64 \
            -machine virt \
            -m 64M \
            -bios none \
            -drive file="$FS_IMG",if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
            -kernel "$KERNEL" \
            -nographic 2>&1 || true
        ;;
    
    *)
        echo -e "${RED}Unknown test mode: $TEST_MODE${NC}"
        echo
        echo "Usage: $0 [MODE]"
        echo
        echo "Modes:"
        echo "  interactive (default) - Interactive QEMU console"
        echo "  ch18_file0 - Test ch18_file0 program"
        echo "  ch18_file1 - Test ch18_file1 program"
        echo "  ch18_file2 - Test ch18_file2 program"
        echo "  ch18_file3 - Test ch18_file3 program"
        exit 1
        ;;
esac
