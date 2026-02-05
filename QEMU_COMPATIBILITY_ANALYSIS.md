# tg-ch18 程序在 Linux QEMU RISC-V64 上的兼容性分析

## 问题分述

当尝试用 `qemu-riscv64` 运行 tg-ch18 的用户程序（如 `ch6_file0`）时：

```bash
$ qemu-riscv64 ./ch6_file0
[ERROR] Panicked at src/bin/ch6_file0.rs:16, assertion failed: fd > 0
```

程序在 `open()` 系统调用处失败。这不是程序的 bug，而是**架构不兼容**的结果。

## 根本原因

### tg-ch18 程序的特性

tg-ch18 下的用户程序（如 ch6_file0, ch6_file1 等）是：

```
编译目标: riscv64gc-unknown-none-elf
链接方式: 静态链接，无 libc
系统调用: 使用 tg-ch18 内核定义的系统调用接口
```

这些程序通过以下方式调用系统调用：

```rust
pub fn open(path: &str, flags: OpenFlags, mode: u32) -> isize {
    unsafe {
        syscall3(
            SyscallId::OPEN,  // <- 这是 tg-ch18 内核的 SyscallId::OPEN
            path.as_ptr() as usize,
            flags.bits as usize,
            mode as usize,
        )
    }
}
```

### 系统调用号的映射

**tg-ch18 系统调用号** (来自 tg-syscall/src/syscall.h.in):
```
#define __NR_open 56
#define __NR_read 63
#define __NR_write 64
#define __NR_close 57
```

**Linux 系统调用号** (RISC-V64):
```
56  = openat (不是 open)
63  = read   (相同)
64  = write  (相同)
57  = close  (相同)
```

### 执行流程分析

当 `qemu-riscv64` 运行程序时：

1. ✅ **加载和执行**: 二进制文件被正确加载
   ```
   ELF 64-bit LSB executable, UCB RISC-V, statically linked
   ```

2. ✅ **用户代码执行**: 程序运行到 `open()` 调用

3. ❌ **系统调用失败**: 
   ```
   程序执行 ecall (a7=56)
   ↓
   QEMU 拦截并映射到 Linux openat() syscall
   ↓
   openat() 收到参数 (dirfd=path指针, pathname=flags, flags=mode, ...)
   ↓
   openat() 返回 -1 (EINVAL: 无效的目录文件描述符)
   ↓
   open() 返回 -1
   ↓
   assert!(fd > 0) 失败，panic!
   ```

## 为什么无法直接在 Linux 上运行

tg-ch18 的程序无法直接在 Linux QEMU 用户模式中运行，主要有三个原因：

### 1. 系统调用号不兼容

| Syscall | tg-ch18 | Linux RISC-V64 | 含义 |
|---------|---------|----------------|------|
| 56 | open(path, flags, mode) | **openat**(dirfd, path, flags) | 参数类型完全不同 |
| 63 | read(fd, buf, count) | read(fd, buf, count) | ✓ 兼容 |
| 64 | write(fd, buf, count) | write(fd, buf, count) | ✓ 兼容 |
| 57 | close(fd) | close(fd) | ✓ 兼容 |

### 2. 链接器入口点不同

Linux 程序期望：
- 从 `_start` 开始执行，由 libc 初始化
- 栈和堆被正确初始化
- 环境变量和命令行参数被传递

tg-ch18 程序：
- 从 `_start` 开始，但没有 libc
- 假设最小的运行环境
- 不依赖标准库初始化

### 3. 文件系统上下文不同

Linux 下：
- 程序继承了当前工作目录
- 文件描述符 0/1/2 (stdin/stdout/stderr) 已打开
- 文件访问通过 Linux VFS

tg-ch18 内核：
- 每个进程有自己的文件描述符表
- stdin/stdout/stderr 可能映射到不同的设备
- 文件访问通过 tg-easy-fs

## 解决方案

### 方案 1: 使用 tg-ch18 内核运行（推荐）

这些程序设计用于在 tg-ch18 内核中运行，这是最佳选择：

```bash
# 在 QEMU 中以虚拟机模式运行 tg-ch18 内核
cargo build --target riscv64gc-unknown-none-elf
# 然后在虚拟机中运行程序
```

**优点**:
- ✅ 程序设计就是为此
- ✅ 所有系统调用都能正常工作
- ✅ 文件系统和进程管理完整

### 方案 2: 为 Linux 重新编译

将用户程序改为编译为 Linux 目标：

```toml
[profile.release]
# 改为
[build]
target = "riscv64gc-unknown-linux-gnu"
```

然后重新编译：
```bash
cargo build --target riscv64gc-unknown-linux-gnu
```

**优点**:
- ✅ 可以在 QEMU 用户模式运行
- ✅ 可以在真实 Linux 系统上运行
- ✅ 使用 standard libc

**缺点**:
- ❌ 需要链接 glibc 或 musl
- ❌ 程序大小会增加
- ❌ 需要修改构建配置

### 方案 3: 创建 QEMU 兼容层

编写一个适配层，在程序启动时检测环境并重新映射系统调用号：

```rust
// 虚拟的兼容层
fn qemu_compatible_syscall(id: SyscallId, args: [usize; 6]) {
    match id {
        SyscallId::OPEN => {
            // tg-ch18: open(path, flags, mode)
            // Linux:  openat(-100, path, flags, mode)
            // 重新映射...
        }
        // ...其他调用
    }
}
```

**优点**:
- ✅ 可在 Linux QEMU 用户模式运行
- ✅ 无需重新编译程序
- ✅ 可用于测试和验证

**缺点**:
- ❌ 相对复杂
- ❌ 并非所有系统调用都能完美映射
- ❌ 需要维护兼容代码

## 验证结果

### 当前状态

```
✅ 二进制格式: 有效的 RISC-V64 ELF
✅ 编译成功: 无编译错误
✅ 程序启动: 成功加载并开始执行
❌ 系统调用: 失败（系统调用号不兼容）
```

### 测试输出

```bash
$ qemu-riscv64 ./ch6_file0
[ERROR] Panicked at src/bin/ch6_file0.rs:16, assertion failed: fd > 0
```

这表明：
1. ✅ QEMU 能加载并执行程序
2. ✅ 程序的用户代码实际上在运行
3. ❌ 系统调用失败，导致 panic

## 建议

对于当前项目的目标（验证 tg-ch18 内核的文件系统和进程管理）：

### ✅ 推荐做法

**在 tg-ch18 内核中运行这些程序**

这是设计初衷，也是正确的验证方式：

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18

# 构建内核
cargo build --target riscv64gc-unknown-none-elf

# 在 QEMU 虚拟机中运行
# 内核在启动时会自动运行 initproc，
# 然后在 shell 中可以手动运行各个程序

# 或者编辑 cases.toml 改变程序加载顺序
```

在内核 shell 中：
```
> ch6_file0
Test file0 OK!

> ch6_file1
Test file1 OK!

> ch6_file2
Test link OK!

> ch6_file3
Test mass open/unlink OK!
```

## 总结

| 场景 | 可行性 | 原因 |
|------|--------|------|
| QEMU 用户模式 (Linux) | ❌ 不可行 | 系统调用号不兼容 |
| tg-ch18 内核 + QEMU | ✅ **可行** | 系统调用号匹配 |
| 原生 Linux (重编译后) | ✅ 可行 | 使用 Linux 系统调用 |

当前 tg-ch18 项目的最佳验证方法是**在 tg-ch18 内核中运行这些程序**。
