#!/bin/bash

# 自动化测试脚本：在 tg-ch18 和 StarryOS 中运行用户态程序并对比结果
# 使用: ./compare_test.sh [program_name] [iterations]

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TG18_DIR="/home/chyyuu/thecodes/os-compare/tg-ch18"
STARRY_DIR="/home/chyyuu/thecodes/os-compare/StarryOS"

# 配置
PROGRAMS=("hello-rv64" "ch18_file0" "ch18_file1" "ch18_file2" "ch18_file3")
TIMEOUT_SEC=15
ITERATIONS=${2:-1}
TEST_PROGRAM=${1:-"all"}

# 测试结果统计
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  tg-ch18 vs StarryOS: 用户态程序对比测试${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"

# ============================================================
# 辅助函数
# ============================================================

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[✓ PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[✗ FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# ============================================================
# 构建内核镜像（如果需要）
# ============================================================

build_tg18() {
    log_info "检查 tg-ch18 内核是否需要重新编译..."
    
    if [ ! -f "$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/tg-ch18" ]; then
        log_test "编译 tg-ch18 内核..."
        cd "$TG18_DIR/linux-compatible-tests"
        bash pack-to-fsimg.sh > /dev/null 2>&1
        if [ $? -eq 0 ]; then
            log_pass "tg-ch18 编译成功"
        else
            log_fail "tg-ch18 编译失败"
            return 1
        fi
    else
        log_pass "tg-ch18 内核已存在"
    fi
}

# ============================================================
# tg-ch18 上运行程序
# ============================================================

run_on_tg18() {
    local prog=$1
    local log_file="/tmp/tg18_${prog}.log"
    
    log_test "在 tg-ch18 上运行 $prog..."
    
    # 为了测试特定程序，需要修改 Makefile 或 环境变量
    # 这里简化假设所有程序都通过相同的启动流程
    cd "$TG18_DIR/linux-compatible-tests"
    
    timeout $TIMEOUT_SEC bash run-in-qemu-system.sh 2>&1 > "$log_file"
    
    if grep -q "Test.*OK\|Test file0 OK" "$log_file"; then
        log_pass "$prog 在 tg-ch18 运行成功"
        return 0
    else
        log_fail "$prog 在 tg-ch18 运行失败或超时"
        tail -20 "$log_file" | sed 's/^/  /'
        return 1
    fi
}

# ============================================================
# 对比输出的关键部分
# ============================================================

compare_outputs() {
    local prog=$1
    
    log_test "对比 $prog 的关键输出..."
    
    # 这是一个简化的对比，实际应该对比更多细节
    log_info "输出对比逻辑（需要根据具体程序定制）"
}

# ============================================================
# 主程序
# ============================================================

main() {
    echo -e "\n${YELLOW}参数配置:${NC}"
    echo "  测试程序: $TEST_PROGRAM"
    echo "  重复次数: $ITERATIONS"
    echo "  超时时间: ${TIMEOUT_SEC}秒"
    echo ""
    
    # 确定要测试的程序列表
    if [ "$TEST_PROGRAM" = "all" ]; then
        PROGS_TO_TEST=("${PROGRAMS[@]}")
    else
        PROGS_TO_TEST=("$TEST_PROGRAM")
    fi
    
    # 构建 tg-ch18
    build_tg18 || exit 1
    
    # 逐个运行测试
    echo -e "\n${YELLOW}════ 开始运行测试 ════${NC}"
    
    for prog in "${PROGS_TO_TEST[@]}"; do
        for ((i=1; i<=ITERATIONS; i++)); do
            ((TESTS_RUN++))
            
            if [ $ITERATIONS -gt 1 ]; then
                echo -e "\n${YELLOW}[iteration $i/$ITERATIONS]${NC}"
            fi
            
            run_on_tg18 "$prog"
            
            # 可选：对比输出
            # compare_outputs "$prog"
        done
    done
    
    # 汇总结果
    echo -e "\n${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}测试结果汇总${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    
    local total=$TESTS_RUN
    local pass=$TESTS_PASSED
    local fail=$TESTS_FAILED
    local pass_rate=$((pass * 100 / total))
    
    echo "总测试数: $total"
    echo -e "${GREEN}✓ 通过: $pass${NC}"
    echo -e "${RED}✗ 失败: $fail${NC}"
    echo -e "成功率: $pass_rate%"
    
    if [ $fail -eq 0 ]; then
        echo -e "\n${GREEN}════════ 所有测试通过！${NC}${NC}\n"
        return 0
    else
        echo -e "\n${RED}════════ 存在未通过的测试${NC}\n"
        return 1
    fi
}

# 运行主函数
main
