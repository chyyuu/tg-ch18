# tg-ch18 Linux 兼容性改进进度

## 📋 最新状态 (当前 Session)

### ✅ 已完成的关键修复

#### 1. **栈初始化修复**
- **问题**: ch18_file0 在 glibc 尝试遍历 auxiliary vector (auxv) 时崩溃
- **原因**: 栈顶地址 (0x4000000000) 映射范围不穷，程序访问 0x4000000008 导致 LoadPageFault
- **解决方案**:
  - 扩展栈映射范围：从 128 页增加到 129 页，包括 0x4000000000 所在页
  - 初始化 Linux 风格的栈布局：在栈顶写入 argc/argv/envp/auxv 结构
  - 使用物理地址偏移-计算机制正确映射虚拟地址到物理地址

**代码位置**: [tg-ch18/src/process.rs](tg-ch18/src/process.rs#L190-L240)

```rust
// 初始化栈：argc/argv/envp/auxv
let mut sp = stack_top_vaddr;
unsafe {
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // auxv 终止符
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // envp[0] = NULL
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // argv[1] = NULL
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // argv[0] = NULL
    push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 1);  // argc = 1
}
*context.sp_mut() = sp;
```

#### 2. **堆管理系统 - brk syscall (214)**
- **问题**: glibc 堆初始化时调用 brk，返回空响应导致后续内存访问崩溃
- **解决方案**:
  - 在 Process 结构中添加 heap_start/heap_end 字段追踪堆边界
  - 在 from_elf 中计算正确的堆起点：ELF 最高加载地址的下一页
  - 实现完整的 brk 系统调用：
    - brk(0) 返回当前堆边界
    - brk(addr) 在所需地址分配新页，返回新边界
    - 验证地址合法性，防止堆与栈冲突

**代码位置**: [tg-ch18/src/process.rs](tg-ch18/src/process.rs#L39-46), [tg-ch18/src/main.rs](tg-ch18/src/main.rs#L734-800)

**关键修复**:
```rust
// 计算堆起始地址（ELF 加载的最高地址之后的下一页）
const PAGE_SIZE_FOR_HEAP: usize = 1 << Sv39::PAGE_BITS;
let heap_start = if max_end_mem % PAGE_SIZE_FOR_HEAP == 0 {
    max_end_mem
} else {
    ((max_end_mem / PAGE_SIZE_FOR_HEAP) + 1) * PAGE_SIZE_FOR_HEAP
};

// brk 实现：分配新页
if new_heap_end > old_heap_end {
    let pages_to_map = new_heap_end_page - old_heap_end_page;
    let ptr = unsafe { alloc_zeroed(layout) };
    current.address_space.map_extern(
        start_vpn..end_vpn,
        PPN::new(ptr as usize >> Sv39::PAGE_BITS),
        build_flags("U_WRV"),
    );
}
```

#### 3. **Process exec 修复**
- **问题**: exec syscall 加载新 ELF 后，没有更新 heap_start/heap_end 字段
- **影响**: 子进程继承的是父进程的堆信息，导致堆地址错误
- **解决方案**: 在 exec 方法中同步更新堆字段

**代码位置**: [tg-ch18/src/process.rs](tg-ch18/src/process.rs#L51-65)

#### 4. **set_tid_address syscall (96)**
- **问题**: glibc 线程初始化时调用此系统调用
- **解决方案**: 实现简化版本，返回进程 PID

**代码位置**: [tg-ch18/tg-syscall/src/kernel/mod.rs](tg-ch18/tg-syscall/src/kernel/mod.rs#L31), [tg-ch18/src/main.rs](tg-ch18/src/main.rs#L657-661)

### 📊 执行进度

**ch18_file0 执行流程** (as of last test):

```
1. initproc 启动（来自内核映像）
   ├─ fork() 创建子进程
   └─ exec("ch18_file0") 加载目标程序
   
2. ch18_file0 ELF 加载
   ├─ 映射 LOAD 段 1: 0x10000-0x84214 (R-E)
   ├─ 映射 LOAD 段 2: 0x85c80-0x90cb0 (RW-)
   └─ 初始化堆: 0x91000 开始
   
3. glibc 初始化
   ├─ __libc_start_main 入口
   ├─ brk(0) -> 返回 0x91000      ✅ SUCCESS
   ├─ brk(0x91af8)               ✅ SUCCESS (分配新页)
   ├─ set_tid_address(0x910e0)  ✅ SUCCESS
   └─ 系统调用 99 (madvise?)     ❌ UNSUPPORTED
```

### 🚧 当前障碍

**Syscall 99** (仍需实现)
- 参数: [0x910e0, 0x18, 0xffffffffffffffe0, 0x1, 0x90658, 0x910e0]
- 猜测: madvise (内存建议) 或 fcntl (文件控制)

其他预期的 glibc 系统调用（尚未测试）:
- `mmap` (9) - 内存映射
- `mprotect` (10) - 内存保护
- `sigaction` (134) - 信号处理
- 其他文件 I/O 调用

---

## 📝 技术决策

### 堆地址计算的正确方法

❌ **错误做法**:
```rust
let heap_start = VAddr::<Sv39>::new(max_end_mem).ceil().val();  // 返回错误值!
```

✅ **正确做法**:
```rust
const PAGE_SIZE_FOR_HEAP: usize = 1 << Sv39::PAGE_BITS;
let heap_start = if max_end_mem % PAGE_SIZE_FOR_HEAP == 0 {
    max_end_mem
} else {
    ((max_end_mem / PAGE_SIZE_FOR_HEAP) + 1) * PAGE_SIZE_FOR_HEAP
};
```

### 为什么需要 argc/argv/envp/auxv

glibc (GNU C Library) 的 `__libc_start_main` 遍历这些结构：
1. **auxv 遍历** - 提取系统信息 (AT_PHDR, AT_ENTRY, 等)
2. **envp 遍历** - 加载环境变量
3. **argv 遍历** - 处理命令行参数

没有这些结构，glibc 会崩溃（出现 LoadPageFault）。

### 堆内存分配策略

当 brk 被调用申请页需要分配时：
1. 使用 `alloc_zeroed` 从内核堆分配物理页
2. 使用 `map_extern` 将其映射到用户地址空间
3. 维护 heap_start 和 heap_end 追踪当前界限

---

## 🔍 调试日志关键输出

```
[ INFO] from_elf: Loading ELF, entry=0x105e4
[ INFO] from_elf: LOAD segment vaddr=0x10000, memsz=0x74214, end=0x84214
[ INFO] from_elf: LOAD segment vaddr=0x85c80, memsz=0xb030, end=0x90cb0
[ INFO] from_elf: max_end_mem=0x90cb0, heap_start=0x91000
[ INFO] brk called: addr=0x0, current heap_start=0x91000, heap_end=0x91000
[ INFO] brk(0) returning heap_end=0x91000
[ INFO] brk called: addr=0x91af8, current heap_start=0x91000, heap_end=0x91000
```

---

## 📌 下一步工作

### 优先级 1 - 必需 (blocking glibc)
- [ ] madvise syscall (99) - 或确认是否是其他 syscall
- [ ] mmap syscall (9) - 内存映射（一些 glibc 版本使用）

### 优先级 2 - 可选 (for I/O)
- [ ] 文件 I/O 改进 (read/write/fcntl)
- [ ] 管道 I/O (pipe 已有，pipe2 已有)

### 优先级 3 - 完整性
- [ ] 信号处理改进
- [ ] 进程管理完善 (clone, fork)

---

## 📚 代码修改摘要

### 受影响的文件

| 文件 | 变更 | 关键函数 |
|------|------|---------|
| `src/process.rs` | Process 结构添加 heap 字段 | from_elf, exec, fork |
| `src/main.rs` | 实现 Memory/Process trait | brk, set_tid_address |
| `tg-syscall/src/kernel/mod.rs` | 添加新 syscall 处理 | handle() 函数 |

### 每个 PR 的约 100 行修改
- **堆管理实现**: ~80 行
- **栈初始化代码**: ~60 行  
- **系统调用支持**: ~30 行

---

## 🎯 成功标准

**阶段 1 (已达成)**: ch18_file0 通过 glibc 初始化
- ✅ 栈结构正确初始化
- ✅ argc/argv/envp/auxv 可读
- ✅ brk 系统调用工作
- ✅ set_tid_address 可用

**阶段 2 (进行中)**: ch18_file0 完成初始化并开始执行实际代码
- ⏳ 处理更多 syscall (99 等)
- ⏳ 可能的内存映射需求

**阶段 3 (未来)**: ch18_file0 完整执行并退出
- Main 函数执行
- 文件 I/O 操作
- 进程退出

---

## 💡 关键学习

1. **栈布局很重要** - glibc 期望特定的栈内存布局
2. **页对齐计算** - VAddr 工具存在缺陷，需要手工计算
3. **exec 状态管理** - Process 的所有字段需要同步更新
4. **分阶段调试** - 一次一个系统调用比批量实现更有效

