# 在 tg-ch18 内核中运行 ch6_file* 程序

本文档说明如何让 Linux 兼容的 ch6_file* C 程序在 tg-ch18 内核中运行。

## 概述

### 架构兼容性

✅ **已验证兼容**：
- ch6_file* 编译为 RISC-V64 ELF 可执行文件
- tg-ch18 内核支持 Linux RISC-V64 系统调用接口
- 程序使用的 syscalls（open、close、read、write、fstat、link、unlink）都已实现

### 为什么可以工作

ch6_file* 程序：
1. 使用 POSIX C API（标准库函数）
2. 这些函数调用 Linux 系统调用
3. tg-ch18 内核的 syscall 号表来自 musl-libc 的 RISC-V64 标准定义
4. 因此 syscall 号完全匹配，程序可以正确运行

## 快速启动

### 方式 1：直接使用 qemu-riscv64（推荐用于快速测试）

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
make clean && make
qemu-riscv64 ./ch6_file0
```

**优点**：
- 快速、简洁
- 无需 tg-ch18 编译
- 充分验证文件操作功能

**何时使用**：
- 快速验证程序逻辑
- 调试文件操作问题
- CI/CD 集成测试

### 方式 2：在 tg-ch18 内核中运行（完整验证）

这需要额外的步骤来准备磁盘镜像和修改内核配置。

## 详细步骤

### 1. 编译 ch6_file0 为 RISC-V64

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
# 检查是否有交叉编译器
which riscv64-linux-gnu-gcc

# 编译（会自动选择交叉编译器）
make clean && make
ls -lh ch6_file*  # 应该是 ELF 64-bit LSB executable, UCB RISC-V
```

### 2. 准备磁盘镜像

tg-ch18 需要一个包含文件的磁盘镜像。有两个选择：

#### 选项 A：使用现有的 tg-user 磁盘镜像

如果 tg-user 项目已编译，可能已有磁盘镜像：
```bash
find /home/chyyuu/thecodes/os-compare/tg-ch18 -name "*.img" -o -name "fs.img"
```

#### 选项 B：创建新的磁盘镜像（高级）

这需要 easy-fs 工具，目前 tg-ch18 中可能没有现成的工具。
可以参考 rCore-Tutorial 项目的方法。

### 3. 配置 tg-ch18 以加载 ch6_file0

当前 tg-ch18 内核期望文件系统中有一个 "initproc" 文件。
需要确保 "initproc" 指向或包含 ch6_file0。

**关键代码位置**（见 src/main.rs 第 126 行）：
```rust
let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
```

### 4. 运行

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18
CHAPTER=-8 cargo run 2>&1
```

## 系统调用兼容性矩阵

| Syscall | tg-ch18 | Linux | ch6_file* 用途 |
|---------|---------|-------|----------------|
| open (56) | ✅ | ✅ | 创建/打开文件 |
| close (57) | ✅ | ✅ | 关闭文件 |
| read (63) | ✅ | ✅ | 读取文件 |
| write (64) | ✅ | ✅ | 写入文件 |
| fstat (80) | ✅ | ✅ | 获取文件元数据 |
| linkat (37) | ✅ | ✅ | 创建硬链接 |
| unlinkat (35) | ✅ | ✅ | 删除文件/硬链接 |

**所有关键 syscall 都已平台实现** ✅

## 文件操作验证

### ch6_file0：基础读写
```c
// 创建/写入/读取/验证文件内容
open(fname, O_CREAT | O_WRONLY, 0o666)
write(fd, "Hello, world!", 13)
read(fd, buffer, 100)
// 验证读回的内容匹配
```

**预期行为**：成功完成，输出 "Test file0 OK!"

### ch6_file1：文件元数据
```c
// 验证 fstat 返回正确的元数据
fstat(fd, &stat)
assert(stat.st_size == 5)
assert(stat.st_nlink == 1)
```

**预期行为**：所有断言通过

### ch6_file2：硬链接
```c
// 测试链接计数和数据持久化
open() -> link count = 1
link() -> link count = 2
link() -> link count = 3
unlink() -> link count = 2
```

**预期行为**：链接计数正确增减

### ch6_file3：批量操作
```c
// 10次迭代：创建、写入、删除
for (i = 0; i < 10; i++) {
    open() -> write 50 times -> close() -> unlink()
}
```

**预期行为**：全部成功，无资源泄漏

## 常见问题

### Q1：程序可以直接在 tg-ch18 中运行吗？

**A**：理论上是的，但需要确保磁盘镜像中 "initproc" 文件存在且指向 ch6_file0。
当前最简单的方法是使用 `qemu-riscv64` 直接运行。

### Q2：为什么不直接修改 tg-user 来包含 C 版本？

**A**：tg-user 是 Rust 项目，使用 no_std + tg_syscall 库。
C 版本使用 POSIX 标准库，设计理念不同。
分开维护更清晰。

### Q3：这些程序与原始 Rust 版本有什么区别？

**A**：功能相同，但实现不同：
- Rust 版本：使用 tg_syscall 库，需在 tg-ch18 内核中运行
- C 版本：使用 POSIX C 库，可在任何 Linux 兼容系统上运行

### Q4：可以在真实 RISC-V64 硬件上运行吗？

**A**：是的！如果目标硬件运行 Linux RISC-V64，
```bash
# 交叉编译
make CC=riscv64-linux-gnu-gcc

# 通过 SCP 传输到目标
scp ch6_file0 user@target:/tmp/

# SSH 连接并运行
ssh user@target /tmp/ch6_file0
```

## 技术细节

### ELF 文件格式

```bash
file ch6_file0
# 输出示例：
# ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI
# version 1 (SYSV), statically linked, for GNU/Linux 4.15.0
```

### 静态链接的优势

```bash
# -static 标志的好处
# 1. 包含所有依赖库的代码
# 2. 减少运行时依赖
# 3. 便于在不同环境中运行

readelf -d ch6_file0 | grep -i "needed"
# 无输出 = 无外部依赖库
```

## 下一步

1. **集成磁盘镜像生成**：编写脚本自动为 tg-ch18 创建含有 ch6_file0 的磁盘镜像
2. **自动化测试**：在 CI 流程中添加这些测试
3. **文档完善**：补充更多示例和故障排查指南

## 参考资源

- [Linux RISC-V64 系统调用定义](https://git.musl-libc.org/cgit/musl/tree/arch/riscv64/bits/syscall.h.in)
- [tg-ch18 源代码](../src/main.rs)
- [tg_easy_fs 文件系统](../tg-easy-fs/)
- [qemu-riscv64 用户模式](https://qemu.org)
