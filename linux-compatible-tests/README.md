# README - tg-ch18 Linux 兼容测试套件

## 🎯 概述

这个目录包含了可以在 **tg-ch18 内核**和**Linux (qemu-riscv64)** 中运行的文件操作测试程序。

程序使用标准 POSIX C APIs，提供三种运行方式：
1. **Linux 原生** - 在你的机器上直接运行
2. **QEMU 用户模式** - `qemu-riscv64` 快速测试
3. **tg-ch18 系统模式** - 完整的内核仿真测试（推荐）

## 🚀 最快开始（3 步）

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests

# 自动完成所有 3 步
./quickstart.sh
```

或手动进行：

### 步骤 1：编译为 RISC-V64
```bash
make clean
CC=riscv64-linux-gnu-gcc make build-ch18-only
```

### 步骤 2：打包到 tg-ch18
```bash
./pack-to-fsimg.sh
```

### 步骤 3：在 QEMU 中运行
```bash
./run-in-qemu-system.sh ch18_file0
```

## 📚 文档导航

| 文档 | 内容 | 适合 |
|------|------|------|
| **[TG_CH18_KERNEL_TESTING.md](TG_CH18_KERNEL_TESTING.md)** | 详细集成指南 | 需要深入了解 |
| **[COMPLETION_SUMMARY.md](COMPLETION_SUMMARY.md)** | 项目完成摘要 | 想了解做了什么 |
| **[TG_CH18_INTEGRATION.md](TG_CH18_INTEGRATION.md)** | 快速参考 | 需要快速查阅 |
| **[QUICKSTART.md](QUICKSTART.md)** | 极简快速开始 | 想快速上手 |

## 📋 程序列表

### ch18_file0 - 基础文件 I/O
创建、写入、读取文件，验证数据完整性
```
./run-in-qemu-system.sh ch18_file0
```
**预期输出**: `Test file0 OK!`

### ch18_file1 - 文件元数据
使用 `fstat()` 获取文件属性（大小、权限、链接计数）
```
./run-in-qemu-system.sh ch18_file1
```
**预期输出**: `Test file1 OK!`

### ch18_file2 - 硬链接管理
创建硬链接、验证链接计数、删除链接
```
./run-in-qemu-system.sh ch18_file2
```
**预期输出**: `Test link OK!`

### ch18_file3 - 批量操作
10 次迭代，每次创建、写入、读取、删除文件
```
./run-in-qemu-system.sh ch18_file3
```
**预期输出**: `Test mass open/unlink OK!`

## 🔧 构建选项

### 编译为 Linux 原生（x86_64）
```bash
make clean && make && ./ch6_file0
```

### 编译为 RISC-V64 用户模式（快速测试）
```bash
make clean && CC=riscv64-linux-gnu-gcc make
qemu-riscv64 ./ch6_file0
```

### 编译为 tg-ch18 系统模式（完整测试）
```bash
make clean && CC=riscv64-linux-gnu-gcc make build-ch18-only
./pack-to-fsimg.sh
./run-in-qemu-system.sh ch18_file0
```

## 📁 文件结构

```
.
├── ch18_file0              ← RISC-V64 可执行文件
├── ch18_file1
├── ch18_file2
├── ch18_file3
├── ch18_file0.c            ← C 源代码
├── ch18_file1.c
├── ch18_file2.c
├── ch18_file3.c
├── ch6_file*.c             ← 符号链接（向后兼容）
├── Makefile                ← 编译配置
├── pack-to-fsimg.sh        ← 打包脚本
├── run-in-qemu-system.sh   ← QEMU 启动脚本
├── quickstart.sh           ← 快速开始向导
├── README.md               ← 本文档
├── QUICKSTART.md           ← 极简开始指南
├── TG_CH18_INTEGRATION.md  ← 集成概述
├── TG_CH18_KERNEL_TESTING.md ← 详细指南
└── COMPLETION_SUMMARY.md   ← 项目摘要
```

## ✅ 前提条件

### 必需
- `riscv64-linux-gnu-gcc` - 跨平台编译器
  ```bash
  sudo apt install gcc-riscv64-linux-gnu
  ```

### 可选但推荐
- `qemu-riscv64` - 用户模式仿真（快速测试）
  ```bash
  sudo apt install qemu-user
  ```

- `qemu-system-riscv64` - 系统模式仿真（完整测试）
  ```bash
  sudo apt install qemu-system-misc
  ```

## 🔄 工作流程

### 快速迭代开发

```bash
# 1. 修改 ch18_file0.c
vi ch18_file0.c

# 2. 重新编译
CC=riscv64-linux-gnu-gcc make build-ch18-only

# 3. 快速用 qemu-riscv64 测试
qemu-riscv64 ./ch6_file0

# 4. 如果通过，打包到 tg-ch18 进行完整测试
./pack-to-fsimg.sh
./run-in-qemu-system.sh ch18_file0
```

### 完整系统验证

```bash
# 生成所有四个程序的 RISC-V64 版本
CC=riscv64-linux-gnu-gcc make build-ch18-only

# 将它们打包到 tg-ch18 内核
./pack-to-fsimg.sh

# 运行所有测试
for i in 0 1 2 3; do
    ./run-in-qemu-system.sh ch18_file$i
    if [ $? -ne 0 ]; then
        echo "ch18_file$i failed!"
        exit 1
    fi
done
```

## 🐛 故障排除

### 问题：`qemu-system-riscv64: command not found`

**解决**：安装 QEMU 系统仿真器
```bash
sudo apt install qemu-system-misc
```

### 问题：`riscv64-linux-gnu-gcc: command not found`

**解决**：安装交叉编译工具链
```bash
sudo apt install gcc-riscv64-linux-gnu
```

### 问题：`pack-to-fsimg.sh 失败`

**检查**：
1. ch18_file* 已编译 - `ls -la ch18_file0`
2. tg-ch18 已构建 - `cd /home/chyyuu/thecodes/os-compare/tg-ch18 && cargo build`

**重试**：
```bash
# 完全清理
make clean
CC=riscv64-linux-gnu-gcc make build-ch18-only

# 重新打包
./pack-to-fsimg.sh
```

### 问题：QEMU 无法启动或内核崩溃

**检查日志**：
```bash
./run-in-qemu-system.sh ch18_file0 2>&1 | head -50
```

**验证文件**：
```bash
file /home/chyyuu/thecodes/os-compare/tg-ch18/target/riscv64gc-unknown-none-elf/debug/tg-ch18
# 应该显示：ELF 64-bit LSB executable, UCB RISC-V, ...
```

## 🎓 学习资源

- **[rCore Tutorial](https://github.com/rcore-os/rCore-Tutorial-in-single-workspace)** - 教程源代码
- **tg-easy-fs** - 简化的文件系统实现
- **tg-syscall** - 系统调用定义和实现
- **Linux RISC-V64 ABI** - 系统调用号标准

## 💡 常见操作

### 查看 Makefile 所有目标
```bash
make help
```

### 仅清理生成的文件（保持源代码）
```bash
make clean
```

### 编译为本地 x86_64
```bash
make clean && make && ./ch6_file0
```

### 编译为 RISC-V64 并用 qemu-riscv64 测试
```bash
CC=riscv64-linux-gnu-gcc make && qemu-riscv64 ./ch6_file0
```

### 为 tg-ch18 编译所有程序
```bash
CC=riscv64-linux-gnu-gcc make build-ch18-only
```

## 📊 系统调用对应

所有程序使用的系统调用都在 tg-ch18 中实现：

| 功能 | Syscall | tg-ch18 号 | Linux 号 |
|------|---------|-----------|---------|
| 创建/打开文件 | `open()` | 56 | 56 |
| 关闭文件 | `close()` | 57 | 57 |
| 读取数据 | `read()` | 63 | 63 |
| 写入数据 | `write()` | 64 | 64 |
| 获取元数据 | `fstat()` | 80 | 80 |
| 创建硬链接 | `link()` | 37 | 37 |
| 删除文件 | `unlink()` | 35 | 35 |

## 📝 许可证

基于 rCore 教程代码，遵循 MIT/Apache 2.0 双许可。

## 🔗 相关项目

- [rCore-Tutorial](https://github.com/rcore-os/rCore-Tutorial-in-single-workspace)
- [tg-user](../tg-user/) - 原始 Rust 程序
- [tg-easy-fs](../tg-easy-fs/) - 文件系统实现
- [tg-syscall](../tg-syscall/) - 系统调用定义

---

**最后更新**：2026-02-05  
**状态**：✅ 完全就绪
