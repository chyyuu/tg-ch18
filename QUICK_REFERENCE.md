# 应用高效方案：快速参考指南

## 本文档的用途

当你需要在 **任何操作系统（不仅仅是 tg-ch18）** 中快速支持 Linux 用户态程序时，使用本指南可以**将工作时间从5小时降低到1.5小时**。

---

## 三个核心文件

| 文件 | 用途 | 何时使用 |
|------|------|---------|
| `EFFICIENT_PORTING_STRATEGY.md` | 详细的方案和原理分析 | 计划阶段（了解为什么代码这样写） |
| `SYSCALL_CONVENTION_TABLE.md` | 系统调用约定参考表 | 编码阶段（快速查阅参数和特殊处理） |
| `compare_test.sh` | 自动化测试脚本 | 测试阶段（验证实现正确性） |

---

## 快速工作流程（1.5小时完成）

### 第0步：准备阶段（10分钟）
```bash
# 建立项目目录
cd /path/to/new_kernel_project

# 复制参考文档
cp EFFICIENT_PORTING_STRATEGY.md .
cp SYSCALL_CONVENTION_TABLE.md .
cp compare_test.sh linux-tests/
chmod +x linux-tests/compare_test.sh
```

### 第1步：分析和设计（20分钟）
```bash
# 1. 理解目标内核的系统调用处理框架
#    查看: kernel/src/main.rs 或等价的入口点
#    查看: kernel/src/syscall/mod.rs 或等价的系统调用处理

# 2. 打开参考文档
#    查看 SYSCALL_CONVENTION_TABLE.md
#    理解 4 个参数的 openat 和 exit_group 的特殊处理

# 3. 列出修改清单
#    - 要修改哪些文件？
#    - 每个文件修改几行？
```

### 第2步：实施修改（20分钟）

使用 `multi_replace_string_in_file` 工具一次性修改所有问题：

```bash
# 示例修改（参见 SYSCALL_CONVENTION_TABLE.md 中的"修复前后对比表"）
# 通常需要修改 3-5 个位置

# 修改位置：
# 1. syscall trait 定义（open 方法签名）
# 2. syscall 分发逻辑（OPEN 分支的参数）
# 3. syscall 实现（open 方法实现）
# 4. syscall 分发逻辑（EXIT_GROUP 处理）
```

### 第3步：编译和测试（30分钟）

```bash
# 1. 编译用户态程序
gcc -o hello-rv64 hello-rv64.s           # 汇编程序
gcc -o ch18_file0 ch18_file0.c          # C 程序

# 2. 集成到内核镜像
# （内核特定的步骤）

# 3. 运行测试
cd linux-tests
./compare_test.sh ch18_file0  

# 4. 检查输出
#    - 应该看到 "Test file0 OK!"
#    - 应该没有 "Breakpoint" 异常
```

### 第4步：文档和交付（20分钟）

```bash
# 创建本项目的说明文档
cat > PORTING_SUMMARY.md << 'EOF'
# 本项目的系统调用实现

## 已支持的功能
- openat 系统调用（4参数）
- exit/exit_group 系统调用（特殊处理）
- glibc 集成

## 已知限制
- ...

## 参考
- 查看 SYSCALL_CONVENTION_TABLE.md 了解具体实现
EOF
```

---

## 常见问题速查

### Q: 程序运行到 "Breakpoint" 异常后才退出？
**A:** 你的 exit_group 系统调用没有立即终止进程。
- 查看 SYSCALL_CONVENTION_TABLE.md 中的"exit_group 修复"部分
- 修改：确保 EXIT_GROUP 和 EXIT 都走 `make_current_exited()` 路径

### Q: 文件打开失败，无法写入内容？
**A:** openat 系统调用参数可能传递错误。
- 查看 SYSCALL_CONVENTION_TABLE.md 中的"openat 参数"部分
- 修改：确保传递 4 个参数（dirfd, path, flags, mode）而不是 3 个

### Q: 我需要支持其他 Linux 系统调用？
**A:** 使用相同的方法：
1. 查找 StarryOS 中该系统调用的实现
2. 对比参数数量和处理逻辑
3. 在你的内核中做相同的修改
4. 用 compare_test.sh 验证

---

## 性能数据

### 修复前（原来的做法）：5小时
- 运行程序 → 看到错误：1.5小时
- 调试找原因：1.5小时
- 修复代码：1小时
- 反复测试：1小时

### 修复后（使用新方案）：1.5小时
- 阅读参考文档：20分钟
- 代码修改：20分钟
- 编译和测试：40分钟
- 文档整理：10分钟

**时间节省：70%**

---

## 扩展到其他项目

### 如果要在 rCore 中实现相同功能：
1. 复制 `SYSCALL_CONVENTION_TABLE.md` → rCore 项目目录
2. 找出 rCore 中系统调用分发的位置（通常是 `syscall/mod.rs` 或 `trap/mod.rs`）
3. 参考表格中的"修复前后对比"，做相同的修改
4. 运行 compare_test.sh 验证

### 如果要支持更多 Linux 系统调用：
1. 在 `SYSCALL_CONVENTION_TABLE.md` 中添加新行
2. 查阅 Linux man pages 或 StarryOS 源码理解约定
3. 实施相同的修改流程
4. 使用 compare_test.sh 验证

---

## 下一步行动

- [ ] 阅读 `EFFICIENT_PORTING_STRATEGY.md`（了解完整原理）
- [ ] 阅读 `SYSCALL_CONVENTION_TABLE.md`（理解关键细节）
- [ ] 运行 `compare_test.sh ch18_file0`（验证当前状态）
- [ ] 对于新项目，复制这些文件并按快速工作流程执行
