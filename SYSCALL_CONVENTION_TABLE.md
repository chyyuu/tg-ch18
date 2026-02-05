# Linux RISC-V64 系统调用约定参考表

## 概述
本表总结了在实现 Linux RISC-V64 用户态程序支持时，需要关注的系统调用约定。
特别针对 tg-ch18 和类似内核，强调与 StarryOS 的差异。

---

## 关键系统调用详解

### 文件操作系列

| 号 | 名称 | 参数 | 返回值 | tg-ch18 状态 | StarryOS状态 | 注意事项 |
|----|------|------|--------|------------|------------|---------|
| 56 | openat | dirfd, path*, flags, mode | fd | ✅ 已修复 | ✅ | **关键**：4个参数，不是open的3个！ |
| 63 | read | fd, buf*, count | 读取字节数 | ✅ | ✅ | 标准实现 |
| 64 | write | fd, buf*, count | 写入字节数 | ✅ | ✅ | 标准实现 |
| 57 | close | fd | 0 | ✅ | ✅ | 标准实现 |
| 72 | fcntl | fd, cmd, arg... | 取决于cmd | ✅ | ✅ | 可变参数 |
| 80 | fstat | fd, statbuf* | 0 | ✅ | ✅ | 标准实现 |

### 进程管理系列

| 号 | 名称 | 参数 | 返回值 | tg-ch18 状态 | StarryOS状态 | 注意事项 |
|----|------|------|--------|------------|------------|---------|
| 93 | exit | code | 无返回 | ✅ | ✅ | **特殊**：立即终止进程，不返回 |
| 94 | exit_group | code | 无返回 | ✅ 已修复 | ✅ | **特殊**：立即终止整个进程组 |
| 114 | wait4 | pid, wstatus*, options, rusage* | pid | ✅ | ✅ | 等待子进程 |

### 内存管理系列

| 号 | 名称 | 参数 | 返回值 | tg-ch18 状态 | StarryOS状态 | 注意事项 |
|----|------|------|--------|------------|------------|---------|
| 12 | brk | addr | 新的heap顶 | ✅ | ✅ | 动态内存管理 |
| 222 | mprotect | addr, length, prot | 0 | ✅ | ✅ | 内存保护标志修改 |
| 222 | mmap | addr, len, prot, flags, fd, offset | addr | ✅ | ✅ | 内存映射 |

### 线程/信号系列

| 号 | 名称 | 参数 | 返回值 | tg-ch18 状态 | StarryOS状态 | 注意事项 |
|----|------|------|--------|------------|------------|---------|
| 99 | set_robust_list | head*, len | 0 | ✅ | ✅ | glibc 线程清理链表 |
| 218 | set_tid_address | tidp* | tidp值 | ✅ | ✅ | glibc 退出通知 |

---

## glibc 集成问题

运行 glibc 编译的 C 程序（如 ch18_file0.c）时，需要处理的关键问题：

### 问题 1：openat 参数
```
C 库调用：open("fname", O_CREAT | O_WRONLY, 0o666)
↓
生成 syscall：openat(-100, "fname", O_CREAT | O_WRONLY, 0o666)
         参数:  a0=-100  a1=文件名   a2=flags           a3=mode
         
错误实现会导致：
  把 -100 (AT_FDCWD宏) 当作路径指针 → 段错误
```

**正确处理**：
- 理解 AT_FDCWD = -100 表示"当前工作目录"
- 实现 dirfd 参数，当 dirfd == -100 时使用当前目录
- 完整的 4 参数：openat(dirfd, path, flags, mode)

### 问题 2：exit_group 的立即退出
```
程序调用：exit(0) 
  ↓ C库转换为 ↓
syscall_exit_group(0)
  ↓ 内核应该 ↓
立即终止整个进程，不返回
  ❌ 如果像普通系统调用一样返回：
     程序继续执行 → 到达 glibc 末尾 ebreak 指令 → Breakpoint 异常
```

**正确处理**：
- EXIT 和 EXIT_GROUP 系统调用是特殊的，调用后不返回
- 在系统调用分发中特别处理（参见 src/main.rs 第 164 行）

### 问题 3：dup/fcntl 的复杂参数
```
fcntl(fd, F_SETFL, flags)
  a0=fd   a1=cmd   a2=flags
  
不同的 cmd 值导致参数含义不同，需要在实现中判断
```

---

## 修复前后对比表

### openat 修复

**修复前（错误）**：
```rust
// tg-syscall/src/kernel/mod.rs
Id::OPEN => {
    IO.call(id, |io| io.open(caller, args[0], args[1], args[2]))
    //                                 ↑ path  ↑ flags  ↑ mode
    //                      只有 3 个参数！
}

// src/main.rs
fn open(&self, _caller: Caller, path: usize, flags: usize, _mode: usize) -> isize {
    //                           ↑ 实际接收的是 dirfd (-100)
    // 导致 path = -100（无效指针）→ 错误
}
```

**修复后（正确）**：
```rust
// tg-syscall/src/kernel/mod.rs
Id::OPEN => {
    IO.call(id, |io| io.open(caller, args[0] as isize, args[1], args[2], args[3]))
    //                                 ↑ dirfd        ↑ path   ↑ flags  ↑ mode
    //                         4 个参数，顺序正确！
}

// src/main.rs
fn open(&self, _caller: Caller, dirfd: isize, path: usize, flags: usize, _mode: usize) -> isize {
    //                           ↑ 正确接收 dirfd (-100)
    // 然后翻译路径（当 dirfd == -100 时使用当前目录）
}
```

### exit_group 修复

**修复前（错误）**：
```rust
// src/main.rs 第 164 行
Ret::Done(ret) => match id {
    Id::EXIT => unsafe { (*processor).make_current_exited(ret) },
    // ❌ EXIT_GROUP 没有特殊处理，被当作普通系统调用
    _ => {
        let ctx = &mut task.context.context;
        *ctx.a_mut(0) = ret as _;  // 只是设置返回值
        unsafe { (*processor).make_current_suspend() };  // 继续执行
    }
}
```

**修复后（正确）**：
```rust
// src/main.rs 第 164 行
Ret::Done(ret) => match id {
    Id::EXIT | Id::EXIT_GROUP => unsafe { (*processor).make_current_exited(ret) },
    // ✅ 两个都立即终止进程
    _ => {
        let ctx = &mut task.context.context;
        *ctx.a_mut(0) = ret as _;
        unsafe { (*processor).make_current_suspend() };
    }
}
```

---

## 推荐的审查清单

在为其他操作系统或内核实现相同功能时，应该：

- [ ] 理解目标 Linux 版本的系统调用约定（参数个数、顺序、特殊处理）
- [ ] 查阅参考实现（如 StarryOS）的对应代码
- [ ] 特别关注"特殊处理"的系统调用（如 exit, exit_group）
- [ ] 测试使用 glibc 编译的程序，而不仅是裸机程序
- [ ] 对比正确内核（如 Linux）和自己实现的输出

---

## 参考资源

- RISC-V ABI 文档
- Linux man pages: syscall(2), openat(2), exit_group(2)
- StarryOS 源码：api/src/syscall/ 目录
- tg-ch18 源码：tg-syscall/src/kernel/mod.rs 和 src/main.rs
