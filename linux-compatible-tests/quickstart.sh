#!/bin/bash
# ==============================================================================
# 快速开始：在 tg-ch18 中运行 ch18_file* 程序（3 步骤）
# ==============================================================================

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  tg-ch18 Linux 兼容文件操作测试 - 快速开始                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 错误处理
error_exit() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# 步骤 1
echo -e "${YELLOW}步骤 1/3: 编译为 RISC-V64...${NC}"
cd "$SCRIPT_DIR"
make clean || error_exit "清理失败"

if ! CC=riscv64-linux-gnu-gcc make build-ch18-only; then
    error_exit "编译失败"
fi
echo -e "${GREEN}✓ 编译完成${NC}"
echo

# 验证编译结果
echo -e "${YELLOW}验证编译结果...${NC}"
for prog in ch18_file0 ch18_file1 ch18_file2 ch18_file3; do
    if [ ! -f "$SCRIPT_DIR/$prog" ]; then
        error_exit "找不到 $prog"
    fi
done
echo -e "${GREEN}✓ 所有程序已编译${NC}"
echo

# 步骤 2
echo -e "${YELLOW}步骤 2/3: 打包到 tg-ch18 文件系统...${NC}"
if ! ./pack-to-fsimg.sh 2>&1 | grep -E "(✓|✅|Error|error)"; then
    error_exit "打包失败"
fi
echo -e "${GREEN}✓ 打包完成${NC}"
echo

# 步骤 3
echo -e "${YELLOW}步骤 3/3：启动 QEMU 系统仿真...${NC}"
echo

echo -e "${BLUE}在 QEMU 的内核中运行测试程序：${NC}"
echo ""
echo "输入以下命令运行测试："
echo "  ls          (列出文件)"
echo "  ./ch18_file0"
echo "  ./ch18_file1"
echo "  ./ch18_file2"
echo "  ./ch18_file3"
echo ""
echo "按 Ctrl-A X 退出 QEMU（或者 Ctrl-A 再按 H 查看帮助）"
echo ""
echo "启动中..."
echo

./test-qemu.sh
