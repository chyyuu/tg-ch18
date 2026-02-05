# tg-ch18 支持完整指南

## 概述

本指南说明如何通过两种方式运行 Linux 兼容的 ch6_file* 程序：

1. **快速方式**：使用 qemu-riscv64（5 秒内完成）
2. **完整方式**：使用 tg-ch18 内核（需要较多配置）

## 快速启动（推荐）

### 方式 A：一行命令快速测试

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
./run-with-tg18.sh quick
```

这会自动：
- ✅ 编译 ch6_file* 程序
- ✅ 检测架构（x86_64 或 RISC-V64）
- ✅ 自动选择运行方式（直接运行或通过 qemu-riscv64）
- ✅ 执行测试并报告结果

**预期输出**：
```
✓ Test passed!
```

### 方式 B：无脚本方式

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests

# 编译
make clean && make

# 运行所有 4 个程序
./ch6_file0 && echo "✅ ch6_file0 passed"
./ch6_file1 && echo "✅ ch6_file1 passed"
./ch6_file2 && echo "✅ ch6_file2 passed"
./ch6_file3 && echo "✅ ch6_file3 passed"

# 或用 Makefile 目标
make test-qemu  # 使用 qemu-riscv64 运行
```

## 完整系统信息

### 新增文件

```
linux-compatible-tests/
├── run-with-tg18.sh (新)          # 自动化启动脚本
├── TG_CH18_INTEGRATION.md (新)    # 完整集成指南
├── Makefile (更新)                # 添加了新的 make 目标
├── README.md (更新)               # 更新了文档
├── ch6_file0
├── ch6_file1
├── ch6_file2
└── ch6_file3
```

### 文件说明

#### 启动脚本 - `run-with-tg18.sh`

**用途**：自动化处理编译、环境检查和测试

**支持的命令**：
```bash
./run-with-tg18.sh check        # 检查环境（Rust、RISC-V target、QEMU）
./run-with-tg18.sh build        # 编译内核和程序
./run-with-tg18.sh test-qemu    # 测试 ch6_file0（QEMU user-mode）
./run-with-tg18.sh test-kernel  # 完整的内核编译和测试
./run-with-tg18.sh setup        # 完整设置（推荐）
./run-with-tg18.sh quick        # 快速编译和测试
./run-with-tg18.sh help         # 显示帮助
```

#### 集成指南 - `TG_CH18_INTEGRATION.md`

**内容**：
- 系统调用兼容性矩阵
- 分步骤详细说明
- 常见问题解答
- 技术深度讨论

**何时查看**：
- 想了解为什么 ch6_file0 能在 tg-ch18 中运行
- 需要完整的磁盘镜像说明
- 想在真实 RISC-V64 硬件上运行

#### 更新的 Makefile

**新增目标**：
```bash
make test-qemu    # 使用 qemu-riscv64 运行所有程序
make test-tg18    # 显示关于 tg-ch18 的信息和说明
```

**原有目标保留**：
```bash
make              # 编译
make clean        # 清理
make help         # 帮助
```

#### 更新的 README.md

**新增内容**：
- tg-ch18 兼容性说明
- Makefile 目标说明
- 系统调用兼容性矩阵
- 完整工作流程示例
- 链接到其他文档

## 实践步骤

### 步骤 1：验证环境

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests

# 检查编译器
which gcc
which riscv64-linux-gnu-gcc  # 可选

# 检查 QEMU（可选）
which qemu-riscv64
which qemu-system-riscv64
```

### 步骤 2：编译程序

```bash
# 自动选择可用的编译器
make clean && make

# 或指定编译器
make clean CC=riscv64-linux-gnu-gcc
```

### 步骤 3：运行测试

**选项 A：本地（最快）**
```bash
# 如果编译的是原生 x86_64 版本
./ch6_file0
```

**选项 B：QEMU RISC-V64**
```bash
# 如果编译的是 RISC-V64 版本
qemu-riscv64 ./ch6_file0

# 或使用 Makefile
make test-qemu
```

**选项 C：自动选择**
```bash
./run-with-tg18.sh quick
```

### 步骤 4：查看详细信息

```bash
# tg-ch18 集成指南
cat TG_CH18_INTEGRATION.md

# 标准 README
cat README.md

# QEMU 兼容性分析
cat ../QEMU_COMPATIBILITY_ANALYSIS.md
```

## 系统调用兼容性验证

所有关键的系统调用都已在 tg-ch18 中实现：

| Syscall | Number | tg-ch18 | Linux RISC-V64 | ch6_file* 使用 |
|---------|--------|---------|----------------|---|
| open | 56 | ✅ | ✅ | ✅ |
| close | 57 | ✅ | ✅ | ✅ |
| read | 63 | ✅ | ✅ | ✅ |
| write | 64 | ✅ | ✅ | ✅ |
| fstat | 80 | ✅ | ✅ | ✅ |
| linkat | 37 | ✅ | ✅ | ✅ |
| unlinkat | 35 | ✅ | ✅ | ✅ |

✅ **100% 兼容** - 所有关键 syscall 号完全匹配

## 高级用法

### 在真实 RISC-V64 硬件上运行

```bash
# 交叉编译
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
make clean CC=riscv64-linux-gnu-gcc

# 传输到目标设备
scp ch6_file0 user@target:/tmp/

# SSH 执行
ssh user@target /tmp/ch6_file0
```

### 使用 tg-ch18 内核

```bash
# 详见 TG_CH18_INTEGRATION.md
# 需要创建磁盘镜像，包含 initproc 文件
cd /home/chyyuu/thecodes/os-compare/tg-ch18
CHAPTER=-8 cargo run 2>&1
```

### 脚本自动化

```bash
# 完整的自动化设置
./run-with-tg18.sh setup

# 这会：
# 1. 检查环境（Rust、RISC-V target）
# 2. 编译 tg-ch18 内核
# 3. 编译 ch6_file 程序
# 4. 使用 qemu-riscv64 运行测试
# 5. 显示下一步提示
```

## 性能对比

| 方式 | 依赖 | 速度 | 验证范围 |
|------|------|------|---------|
| 本地 x86_64 | 只需 gcc | ⚡ <1s | 文件操作 |
| QEMU user-mode | qemu-riscv64 | ⚡ 2-5s | 文件操作 + RISC-V ISA |
| tg-ch18 内核 | Rust + cargo | 🐢 1-2min | 完整系统 + 内核功能 |

## 故障排查

### 问题 1：找不到编译器

```bash
# 检查 gcc
which gcc
# 如果没有：sudo apt install build-essential

# 检查 RISC-V 编译器
which riscv64-linux-gnu-gcc
# 如果没有：sudo apt install gcc-riscv64-linux-gnu
```

### 问题 2：ch6_file1 失败 - File size mismatch

```bash
# 原因：前一次测试的文件还在
rm -f fname* linkname*
./ch6_file1
```

### 问题 3：qemu-riscv64 找不到

```bash
# 安装 QEMU 用户模式
sudo apt install qemu-user
which qemu-riscv64
```

### 问题 4：权限不足

```bash
# 确保当前目录可写
cd /tmp
cp -r /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests .
cd linux-compatible-tests
make && ./ch6_file0
```

## 文档地图

```
tg-ch18/
├── linux-compatible-tests/
│   ├── TG_CH18_INTEGRATION.md  ← 如何在 tg-ch18 内核中运行
│   ├── run-with-tg18.sh        ← 自动化脚本
│   ├── README.md               ← 程序文档
│   └── Makefile                ← 编译配置
├── QUICK_START.md              ← 快速开始指南（整个项目）
├── QEMU_COMPATIBILITY_ANALYSIS.md ← 系统调用兼容性分析
├── LINUX_COMPATIBILITY_TEST_REPORT.md ← 详细测试结果
└── src/main.rs                 ← 内核源代码
```

## 相关命令速查

```bash
# 编译
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
make                              # 自动选择编译器
make CC=gcc                       # 强制 x86_64
make CC=riscv64-linux-gnu-gcc    # 强制 RISC-V64

# 运行
./ch6_file0                       # 运行单个程序
qemu-riscv64 ./ch6_file0        # 在 QEMU 中运行
make test-qemu                   # 运行所有程序，QEMU

# 自动化
./run-with-tg18.sh quick         # 快速测试
./run-with-tg18.sh setup         # 完整设置
./run-with-tg18.sh check         # 环境检查

# 信息
cat TG_CH18_INTEGRATION.md        # 集成指南
cat README.md                     # 程序文档
make help                         # Makefile 帮助
./run-with-tg18.sh help          # 脚本帮助
```

## 总结

✅ **已实现的功能**：
- ch6_file* 可以在 Linux 上直接运行
- ch6_file* 可以在 QEMU user-mode RISC-V64 上运行
- ch6_file* 在 tg-ch18 内核中理论上可运行（需磁盘镜像）
- 完整的文档和自动化脚本

🎯 **下一步**：
1. 运行 `./run-with-tg18.sh quick` 快速验证
2. 查看 `TG_CH18_INTEGRATION.md` 了解详情
3. 根据需要在 tg-ch18 内核或真实硬件上部署

💡 **推荐用途**：
- **快速验证**：使用 qemu-riscv64（推荐）
- **完整验证**：使用 tg-ch18 内核
- **生产部署**：在实际 RISC-V64 Linux 系统上运行
