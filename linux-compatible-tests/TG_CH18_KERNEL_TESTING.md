# 在 tg-ch18 中运行 ch18_file* - 完整指南

## 概述

此目录现在包含 **三种运行方式**来验证文件操作功能：

1. **qemu-riscv64 用户模式**（最快）- `./ch6_file*`
2. **tg-ch18 内核模式**（本指南）- `./ch18_file*`
3. **Linux 原生**（参考）- `./ch6_file*`

本指南关注 **方式 2：在 tg-ch18 系统中运行**。

## 什么是 ch18_file*？

`ch18_file*.c` 是 `ch6_file*.c` 的重命名版本，专门为在 tg-ch18 内核中运行而优化：

- 相同的功能和测试逻辑
- 编译为 RISC-V64 ELF 格式
- 打包到 tg-ch18 的 Easy-FS 文件系统镜像
- 由 tg-ch18 内核通过 syscall 装载和执行

## 快速开始 (3 步)

### 步骤 1：编译为 RISC-V64

```bash
cd linux-compatible-tests
make clean
CC=riscv64-linux-gnu-gcc make build-ch18-only
```

验证：
```bash
file ch18_file0
# 输出应为: ELF 64-bit LSB executable, UCB RISC-V, ...
 
ls -lh ch18_file*
# 应该显示 4 个可执行文件
```

### 步骤 2：打包到 tg-ch18 文件系统

```bash
./pack-to-fsimg.sh
```

这个脚本自动：
- ✅ 验证二进制文件
- ✅ 复制到 `tg-ch18/linux-user/`
- ✅ 更新 `cases.toml` 
- ✅ 重建 tg-ch18 内核（带打包的二进制文件）

**输出应如下所示：**
```
=== Packing ch18_file* into tg-ch18 filesystem ===
...
✓ tg-ch18 build successful
Kernel: /home/chyyuu/thecodes/os-compare/tg-ch18/target/riscv64gc-unknown-none-elf/debug/tg-ch18 (6.7M)
Filesystem: /home/chyyuu/thecodes/os-compare/tg-ch18/target/riscv64gc-unknown-none-elf/debug/fs.img (64M)
```

### 步骤 3：在 QEMU 中启动

```bash
./run-in-qemu-system.sh
```

这将启动 QEMU 交互模式。在 QEMU 的 RISC-V64 virt 虚拟机中，你可以运行：

```
# 在 QEMU 提示符输入
./ch18_file0
./ch18_file1
./ch18_file2
./ch18_file3
```

**预期输出：**
```
Test file0 OK!
Test file1 OK!
Test link OK!
Test mass open/unlink OK!
```

## 无交互启动单个程序

### 选项 1：自动测试循环

```bash
for i in 0 1 2 3; do
    echo "Testing ch18_file$i..."
    ./run-in-qemu-system.sh "ch18_file$i"
done
```

### 选项 2：仅运行 ch18_file0

```bash
./run-in-qemu-system.sh ch18_file0
```

这会启动内核并自动运行 `ch18_file0`，然后显示结果。

## 系统架构

```
┌─────────────────────────────────────────────┐
│   QEMU System Mode (qemu-system-riscv64)   │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────────────────────────────────┐  │
│  │    tg-ch18 Kernel                     │  │
│  │  (ELF executable)                     │  │
│  │  - Entry point: kernel setup          │  │
│  │  - Loads: initproc, ch18_file0-3      │  │
│  │  - Implements: All required syscalls  │  │
│  └──────────────────────────────────────┘  │
│                     ↓                       │
│  ┌──────────────────────────────────────┐  │
│  │    Easy-FS Filesystem                 │  │
│  │  (fs.img, 64M block device)           │  │
│  │  - Root inode                         │  │
│  │  - Files: initproc, ch18_file0-3      │  │
│  │  - Directories supported              │  │
│  └──────────────────────────────────────┘  │
│                                             │
└─────────────────────────────────────────────┘
      ↓
   Virtual virt Machine
   - CPU: RISC-V64
   - RAM: 64 MB
   - RTC: QEMU virt machine timer
   - Block device: fs.img
```

## 关键系统调用支持

tg-ch18 实现的必要 syscalls：

| Syscall | Number | Purpose | 实现状态 |
|---------|--------|---------|--------|
| `open` | 56 | 创建/打开文件 | ✅ |
| `close` | 57 | 关闭文件 | ✅ |
| `read` | 63 | 读取数据 | ✅ |
| `write` | 64 | 写入数据 | ✅ |
| `fstat` | 80 | 获取文件元数据 | ✅ |
| `link` | 37 | 创建硬链接 | ✅ |
| `unlink` | 35 | 删除文件 | ✅ |
| `exit` | 93 | 进程退出 | ✅ |
| `getpid` | 172 | 获取进程 ID | ✅ |

所有系统调用号都来自 musl-libc 的 RISC-V64 标准定义。

## 调试技巧

### 1. 查看内核日志

QEMU 在启动时会打印日志：

```
[INFO] Initializing console...
[INFO] Loading filesystem...
[INFO] Starting initproc...
[INFO] Executing user program...
```

### 2. 检查文件大小

```bash
ls -lh /home/chyyuu/thecodes/os-compare/tg-ch18/target/riscv64gc-unknown-none-elf/debug/
# tg-ch18: 内核可执行文件
# fs.img: 文件系统镜像 (通常 64MB)
```

### 3. 清理和重建

```bash
# 完全清理
make clean

# 重新编译
CC=riscv64-linux-gnu-gcc make build-ch18-only

# 重新打包
./pack-to-fsimg.sh

# 重新启动
./run-in-qemu-system.sh ch18_file0
```

### 4. 修改程序后的步骤

如果你修改了 `.c` 文件：

```bash
# 重新编译（会生成新的 ELF）
CC=riscv64-linux-gnu-gcc make build-ch18-only

# 重新打包（会重建 tg-ch18）
./pack-to-fsimg.sh

# 测试新版本
./run-in-qemu-system.sh ch18_file_X
```

## 与其他方式的对比

### qemu-riscv64 用户模式（快速测试）

```bash
# Quick verification
make clean && make && qemu-riscv64 ./ch6_file0
```

**优点**：
- ⚡ 非常快速
- 不需要 tg-ch18 编译
- 使用真实的 Linux syscalls

**缺点**：
- 不在 tg-ch18 kernel 中
- 文件系统在主机 /tmp
- 不会测试 Easy-FS

### tg-ch18 系统模式（本指南）

```bash
CC=riscv64-linux-gnu-gcc make build-ch18-only
./pack-to-fsimg.sh
./run-in-qemu-system.sh ch18_file0
```

**优点**：
- ✅ 验证 tg-ch18 兼容性
- ✅ 用 Easy-FS 测试文件系统
- ✅ 完整的系统模拟

**缺点**：
- 需要重新构建 tg-ch18
- 速度较慢（10-30 秒）

### Linux 原生（参考）

```bash
# On native Linux
make clean && gcc -o ch6_file0 ch18_file0.c && ./ch6_file0
```

**用途**：
- 验证 C 代码逻辑
- 测试主机 Linux 兼容性
- 性能基准

## 常见问题

### Q: 为什么需要重命名为 ch18_file*？

**A**: 这使得：
1. 清晰的目的（为 tg-ch18 设计）
2. 不与 ch6_file*（backward compat）混淆
3. cases.toml 条目有明确的命名

### Q: 能否跳过 pack-to-fsimg.sh？

**A**: 否。该脚本是必需的，因为它：
1. 将二进制文件放到 tg-ch18/linux-user/
2. 更新 cases.toml
3. 重新编译 tg-ch18 将文件打包到 fs.img

### Q: Easy-FS 与 Linux ext4 有何不同？

**A**: Easy-FS 是一个简化的文件系统：
- 结构更简单，适合教学
- 支持基本操作（open/close/read/write）
- 支持文件元数据（size, mode, nlink）
- 支持目录导航
- 块大小：512 字节

## 下一步

1. **修改程序**：编辑 `ch18_file*.c`，添加新的测试
2. **添加更多 syscall**：实现 `mkdir()`、`chdir()` 等
3. **性能分析**：比较 qemu-riscv64 vs tg-ch18 的 I/O 性能
4. **集成测试**：将脚本集成到 CI/CD 流程

## 相关文件

- [Makefile](Makefile) - 编译配置
- [ch18_file0.c](ch18_file0.c) - 基础文件 I/O 测试
- [ch18_file1.c](ch18_file1.c) - 文件元数据测试
- [ch18_file2.c](ch18_file2.c) - 硬链接测试
- [ch18_file3.c](ch18_file3.c) - 批量操作测试
- [pack-to-fsimg.sh](pack-to-fsimg.sh) - 打包脚本
- [run-in-qemu-system.sh](run-in-qemu-system.sh) - 启动脚本

## 许可证

基于 rCore 教程代码，遵循 MIT/Apache 2.0 许可。
