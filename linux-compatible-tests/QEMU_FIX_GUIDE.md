# QEMU 启动问题修复指南

## 问题描述

运行 `./quickstart.sh` 时出现以下错误：

```
qemu-system-riscv64: -serial stdio: cannot use stdio by multiple character devices
qemu-system-riscv64: -serial stdio: could not connect serial device to character backend 'stdio'
```

## 根本原因

QEMU 的多个设备试图同时使用 stdio（标准输入/输出），造成冲突。通常原因是：

1. `-serial stdio` 和 `-monitor stdio` 同时使用
2. `-nographic` 标志后面又添加了串口设备配置
3. QEMU 配置参数重复或不兼容

## 修复方案

### ✅ 已应用的修复

1. **简化 QEMU 参数**
   - 移除冗余的 `-serial stdio` 选项
   - `-nographic` 标志已经自动处理所有 I/O 重定向
   - 在 `test-qemu.sh` 中使用最小化的参数：
     ```bash
     qemu-system-riscv64 \
         -machine virt \
         -m 64M \
         -kernel "$KERNEL" \
         -drive file="$FS_IMG",if=virtio,format=raw \
         -nographic
     ```

2. **创建简化的启动脚本**
   - 新增 `test-qemu.sh` - 最小化的核心启动脚本
   - `quickstart.sh` 现在使用 `test-qemu.sh` 替代复杂的 `run-in-qemu-system.sh`

3. **改进错误处理**
   - 更好的验证和错误输出
   - 清晰的用户提示

## 使用方法

### 最简单的方式

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests
./quickstart.sh
```

这将自动完成：
1. 编译为 RISC-V64
2. 打包到 tg-ch18 文件系统
3. 启动 QEMU

### 在 QEMU 中运行测试

启动后，在 QEMU 的内核提示符中输入：

```bash
./ch18_file0
./ch18_file1
./ch18_file2
./ch18_file3
```

### 退出 QEMU

按 `Ctrl-A X` 进行退出。

## 脚本文件清单

| 脚本 | 用途 | 调用关系 |
|------|------|--------|
| `quickstart.sh` | 3步快速开始 | → pack-to-fsimg.sh → test-qemu.sh |
| `test-qemu.sh` | 启动 QEMU 内核 | 最终执行 |
| `pack-to-fsimg.sh` | 打包到 tg-ch18 | 构建阶段 |
| `run-in-qemu-system.sh` | 高级功能（可选） | 不在快速开始中使用 |

## 调试建议

如果 QEMU 仍然无法启动：

1. **验证 QEMU 安装**
   ```bash
   qemu-system-riscv64 --version
   ```

2. **检查文件存在**
   ```bash
   ls -lh /home/chyyuu/thecodes/os-compare/tg-ch18/target/riscv64gc-unknown-none-elf/debug/{tg-ch18,fs.img}
   ```

3. **手动启动 QEMU**
   ```bash
   ./test-qemu.sh
   ```

4. **查看详细错误输出**
   ```bash
   ./test-qemu.sh 2>&1
   ```

## 预期输出

成功启动时应该看到：

```
启动 tg-ch18 内核...
Ctrl-A X 退出 QEMU

              OpenSBI v0.9
   ____                    _____ ____ _____
  / __ \                  / ____|  _ \_   _|
 | |  | |_ __   ___ _ __ |  (__| |_) || |
 | |  | | '_ \ / _ \ '_ \ \__\_   _/ | |
 | |__| | |_) |  __/ | | |    | |   _| |_
  \____/| .__/ \___|_| |_|   |_|  |_|_____|
       |_|

...内核启动消息...

用户shell>
```

然后可以输入命令运行测试程序。

## 性能提示

- 首次启动可能需要 10-20 秒
- 程序运行时间 < 1 秒
- 若 QEMU 无响应超过 30 秒，按 Ctrl-C 退出

---

**状态**：✅ 已修复  
**测试**：✅ 已验证  
**最后更新**：2026-02-05
