#!/bin/bash
# =============================================================================
# tg-ch18 Debug 日志测试脚本
# 运行 ch18_file0 并显示详细的 syscall debug 日志
# =============================================================================

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  tg-ch18 Syscall Debug 日志测试                            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 确保在 tg-ch18 目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 编译 debug 版本
echo -e "${YELLOW}步骤 1: 编译 debug 版本...${NC}"
if ! cargo build 2>&1 | tail -3; then
    echo -e "${RED}编译失败${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 编译完成${NC}"
echo ""

# 运行 QEMU（自动 5 秒后超时）
echo -e "${YELLOW}步骤 2: 启动 QEMU 并查看 debug 日志...${NC}"
echo -e "${BLUE}提示：日志中绿色的 [DEBUG] 行显示 syscall 入口信息${NC}"
echo ""

timeout 5 qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -bios none \
    -drive file="target/riscv64gc-unknown-none-elf/debug/fs.img",if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -kernel "target/riscv64gc-unknown-none-elf/debug/tg-ch18" \
    -nographic

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  测试完成！                                                 ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "可以看到每个 syscall 的调用都被记录了，包括："
echo "  - sys_fork, sys_exec, sys_wait"
echo "  - sys_read, sys_write, sys_open, sys_close"
echo "  - sys_brk, sys_getrandom, sys_mprotect"
echo "  - 等等..."
echo ""
echo "如需查看更详细的输出，可以移除 timeout 限制："
echo "  timeout 30 qemu-system-riscv64 ... (替换为更长的超时时间)"
