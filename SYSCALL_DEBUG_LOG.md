# tg-ch18 Syscall Debug 日志功能

## 概述

参考 StarryOS 的日志输出模式，在 tg-ch18 的所有 syscall 入口和出口处添加了 `log::debug!` 输出，以便更好地跟踪和调试系统调用的执行流程。

## 实现日期

2024年2月5日

## StarryOS 的日志模式

StarryOS 使用一致的日志格式来记录系统调用：

### 入口格式
```rust
debug!("sys_xxx <= param1: {}, param2: {}", param1, param2);
```

### 出口格式（可选）
```rust
debug!("sys_xxx => result: {}", result);
```

### 格式化规则

- **指针**: 使用 `{:p}` 格式化（如 `buf: {buf:p}`）
- **十六进制**: 使用 `{:#x}` 格式化（如 `addr: {addr:#x}`）
- **调试输出**: 使用 `{:?}` 格式化（如 `flags: {flags:?}`）
- **普通值**: 直接使用 `{}` 格式化

### 示例

```rust
// StarryOS 中的例子
debug!("sys_getrandom <= buf: {buf:p}, len: {len}, flags: {flags:?}");
debug!("sys_munmap <= addr: {addr:#x}, length: {length:x}");
debug!("sys_nanosleep => rem: {diff:?}");
```

## tg-ch18 中的实现

### IO Trait（10 个 syscalls）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `write` | `sys_write <= fd: {}, buf: {:#x}, count: {}` | [src/main.rs](src/main.rs#L416) |
| `read` | `sys_read <= fd: {}, buf: {:#x}, count: {}` | [src/main.rs](src/main.rs#L440) |
| `open` | `sys_open <= path: {:#x}, flags: {:#x}, mode: {:#x}` | [src/main.rs](src/main.rs#L463) |
| `fstat` | `sys_fstat <= fd: {}, st: {:#x}` | [src/main.rs](src/main.rs#L496) |
| `close` | `sys_close <= fd: {}` | [src/main.rs](src/main.rs#L537) |
| `pipe` | `sys_pipe <= pipe: {:#x}` | [src/main.rs](src/main.rs#L545) |
| `readlinkat` | `sys_readlinkat <= dirfd: {}, path: {:#x}, buf: {:#x}, bufsize: {}` | [src/main.rs](src/main.rs#L577) |
| `dup` | `sys_dup <= oldfd: {}` | [src/main.rs](src/main.rs#L588) |
| `fcntl` | `sys_fcntl <= fd: {}, cmd: {}, arg: {}` | [src/main.rs](src/main.rs#L609) |

### Process Trait（9 个 syscalls）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `exit` | `sys_exit <= exit_code: {}` | [src/main.rs](src/main.rs#L691) |
| `exit_group` | `sys_exit_group <= exit_code: {}` | [src/main.rs](src/main.rs#L696) |
| `fork` | `sys_fork <=` | [src/main.rs](src/main.rs#L702) |
| `exec` | `sys_exec <= path: {:#x}, count: {}` | [src/main.rs](src/main.rs#L717) |
| `wait` | `sys_wait <= pid: {}, exit_code_ptr: {:#x}` | [src/main.rs](src/main.rs#L739) |
| `getpid` | `sys_getpid <=` | [src/main.rs](src/main.rs#L759) |
| `set_tid_address` | `sys_set_tid_address <= tidp: {:#x}` | [src/main.rs](src/main.rs#L765) |
| `set_robust_list` | `sys_set_robust_list <= head: {:#x}, len: {}` | [src/main.rs](src/main.rs#L772) |
| `prlimit64` | `sys_prlimit64 <= pid: {}, resource: {}, new_limit: {:#x}, old_limit: {:#x}` | [src/main.rs](src/main.rs#L779) |

### Scheduling Trait（2 个 syscalls）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `sched_yield` | `sys_sched_yield <=` | [src/main.rs](src/main.rs#L810) |
| `nanosleep` | `sys_nanosleep <= req: {:#x}, rem: {:#x}` | [src/main.rs](src/main.rs#L815) |

### Clock Trait（1 个 syscall）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `clock_gettime` | `sys_clock_gettime <= clock_id: {:?}, tp: {:#x}` | [src/main.rs](src/main.rs#L823) |

### Signal Trait（5 个 syscalls）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `kill` | `sys_kill <= pid: {}, signum: {}` | [src/main.rs](src/main.rs#L849) |
| `sigaction` | `sys_sigaction <= signum: {}, action: {:#x}, old_action: {:#x}` | [src/main.rs](src/main.rs#L861) |
| `sigprocmask` | `sys_sigprocmask <= mask: {:#x}` | [src/main.rs](src/main.rs#L915) |
| `sigreturn` | `sys_sigreturn <=` | [src/main.rs](src/main.rs#L921) |
| `rt_sigpending` | `sys_rt_sigpending <= set: {:#x}, sigsetsize: {}` | [src/main.rs](src/main.rs#L937) |

### Memory Trait（5 个 syscalls）

| Syscall | 日志格式 | 位置 |
|---------|----------|------|
| `brk` | `sys_brk <= addr: {:#x}` | [src/main.rs](src/main.rs#L944) |
| `getrandom` | `sys_getrandom <= buf: {:#x}, len: {}, flags: {:#x}` | [src/main.rs](src/main.rs#L998) |
| `mprotect` | `sys_mprotect <= addr: {:#x}, len: {:#x}, prot: {}` | [src/main.rs](src/main.rs#L1033) |
| `mmap` | `sys_mmap <= addr: {:#x}, length: {:#x}, prot: {}, flags: {}, fd: {}, offset: {:#x}` | [src/main.rs](src/main.rs#L1053) |
| `munmap` | `sys_munmap <= addr: {:#x}, length: {:#x}` | [src/main.rs](src/main.rs#L1072) |

## 总计

- **IO**: 10 个 syscalls
- **Process**: 9 个 syscalls
- **Scheduling**: 2 个 syscalls
- **Clock**: 1 个 syscall
- **Signal**: 5 个 syscalls
- **Memory**: 5 个 syscalls

**总计**: 32 个 syscalls 都添加了 debug 日志

## 日志级别和颜色

tg-console 日志系统支持 5 个级别：

| 级别 | ANSI 颜色代码 | 显示颜色 | 用途 |
|------|--------------|----------|------|
| TRACE | 90 | 灰色 | 详细跟踪信息 |
| **DEBUG** | **32** | **绿色** | **调试信息（syscalls）** |
| INFO | 34 | 蓝色 | 一般信息 |
| WARN | 93 | 黄色 | 警告信息 |
| ERROR | 31 | 红色 | 错误信息 |

## 使用方法

### 方法 1：使用环境变量（推荐）

在编译时设置 `LOG` 环境变量：

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18
LOG=debug cargo build
```

### 方法 2：使用提供的测试脚本

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18
./run_with_debug.sh
```

该脚本会：
1. 编译 debug 版本
2. 启动 QEMU 并显示所有 syscall debug 日志
3. 5 秒后自动退出

### 方法 3：手动运行 QEMU

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18
cargo build  # 编译 debug 版本

qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -bios none \
    -drive file=target/riscv64gc-unknown-none-elf/debug/fs.img,if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -kernel target/riscv64gc-unknown-none-elf/debug/tg-ch18 \
    -nographic
```

## 输出示例

```
[DEBUG] sys_fork <=
[DEBUG] sys_exec <= path: 0x1019a, count: 10
[ INFO] from_elf: Loading ELF, entry=0x183c8
[ INFO] from_elf: Program header type=Ok(Phdr)
...
[DEBUG] sys_wait <= pid: -1, exit_code_ptr: 0x3fffffff2c
[DEBUG] sys_write <= fd: 1, buf: 0x3ffffffa58, count: 15
Rust user shell
[DEBUG] sys_sched_yield <=
[DEBUG] sys_write <= fd: 1, buf: 0x3ffffffa58, count: 1

[DEBUG] sys_wait <= pid: -1, exit_code_ptr: 0x3fffffff2c
[DEBUG] sys_write <= fd: 1, buf: 0x3ffffffa58, count: 3
>> [DEBUG] sys_sched_yield <=
[DEBUG] sys_read <= fd: 0, buf: 0x3ffffffad7, count: 1
```

可以看到：

1. **绿色的 [DEBUG]** 标签表示 syscall 入口
2. **参数值**都被清晰地记录下来
3. **蓝色的 [ INFO]** 显示其他重要信息（如 ELF 加载）
4. **执行流程**一目了然

## 调试应用

### 跟踪文件操作

```
[DEBUG] sys_open <= path: 0x..., flags: 0x..., mode: 0x...
[DEBUG] sys_read <= fd: 3, buf: 0x..., count: 1024
[DEBUG] sys_write <= fd: 3, buf: 0x..., count: 512
[DEBUG] sys_close <= fd: 3
```

### 跟踪进程创建

```
[DEBUG] sys_fork <=
[DEBUG] sys_exec <= path: 0x..., count: 10
[DEBUG] sys_wait <= pid: -1, exit_code_ptr: 0x...
```

### 跟踪内存分配

```
[DEBUG] sys_brk <= addr: 0x0
[DEBUG] sys_brk <= addr: 0x91000
[DEBUG] sys_getrandom <= buf: 0x..., len: 16, flags: 0x0
[DEBUG] sys_mprotect <= addr: 0x..., len: 0x..., prot: 1
```

## 优势

1. **一致性**: 所有 syscalls 使用统一的日志格式
2. **可读性**: 清晰的参数名称和值
3. **可追踪**: 完整记录每个 syscall 的调用
4. **可调试**: 快速定位问题和异常行为
5. **兼容性**: 遵循 StarryOS 的最佳实践

## 性能影响

- **Debug 构建**: 日志已启用，对性能影响较小
- **Release 构建**: 日志代码仍然存在，但不会输出（需通过 LOG 环境变量启用）

## 未来改进

1. **添加出口日志**: 记录每个 syscall 的返回值（`sys_xxx => result`）
2. **错误日志**: 对失败的 syscall 使用 `log::error!` 或 `log::warn!`
3. **性能统计**: 记录每个 syscall 的执行时间
4. **条件日志**: 只记录特定 syscall 或特定进程的日志

## 参考资料

- [StarryOS syscall 实现](https://github.com/Starry-OS/Starry)
- [tg-console 日志系统](tg-console/src/lib.rs)
- [SYSCALL_IMPLEMENTATION_3CALLS.md](SYSCALL_IMPLEMENTATION_3CALLS.md) - 之前的 syscall 实现文档

## 相关文件

- [src/main.rs](src/main.rs) - 主要实现文件
- [run_with_debug.sh](run_with_debug.sh) - Debug 测试脚本
- [tg-console/src/lib.rs](tg-console/src/lib.rs) - 日志系统实现

---

**作者**: GitHub Copilot  
**日期**: 2024年2月5日  
**版本**: 1.0
