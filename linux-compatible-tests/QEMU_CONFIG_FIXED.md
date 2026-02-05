# QEMU 配置修复报告

## 问题汇总

在运行 `./quickstart.sh` 时遇到了两个关键问题：

### 问题 1：ROM 地址冲突 (已修复 ✅)

**错误**：
```
qemu-system-riscv64: Some ROM regions are overlapping
/home/chyyuu/thecodes/install-qemu-10.2.0/bin/../share/qemu/opensbi-riscv64-generic-fw_dynamic.bin
(addresses 0x0000000080000000 - 0x0000000080042a98) overlaps with
tg-ch18 ELF program header segment 0 
(addresses 0x0000000080000000 - 0x00000000800000ae)
```

**根本原因**：OpenSBI 固件和内核尝试加载到同一地址

**解决方案**：
- 添加 `-bios none` 参数禁用默认 OpenSBI 固件加载
- tg-ch18 内核可以独立启动，不需要第三方 BIOS

### 问题 2：Virtio 块设备初始化失败 (已修复 ✅)

**错误**：
```
panicked at src/virtio_block.rs:19:22:
Error when creating MmioTransport: ZeroDeviceId
```

**根本原因**：QEMU virtio 设备配置不正确，设备没有连接到正确的 MMio 总线

**正确的 QEMU 参数**：
```bash
-drive file="$FS_IMG",if=none,format=raw,id=x0
-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
```

**关键要素**：
- 驱动器使用 `if=none` 而不是 `if=virtio`
- 设备必须显式指定 `bus=virtio-mmio-bus.0`
- 这样才能正确连接到 virt 机器的虚拟 I/O 总线

这个配置来自 tg-ch18 官方的 `.cargo/config.toml` 文件。

## 修复前后对比

### 修复前（错误的配置）
```bash
qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -kernel "$KERNEL" \
    -drive file="$FS_IMG",if=virtio,format=raw \
    -nographic
```

**结果**：
- ❌ ROM 地址冲突导致启动失败
- ❌ 即使禁用块设备也会 panic

### 修复后（正确的配置）
```bash
qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -bios none \
    -drive file="$FS_IMG",if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -kernel "$KERNEL" \
    -nographic
```

**结果**：
- ✅ 内核成功启动
- ✅ 块设备正确识别和初始化
- ✅ 用户程序可以运行

## 启动性能

```
[ INFO] MMIO range -> 0x10001000..0x10002000
[ INFO] device features: SEG_MAX | GEOMETRY | BLK_SIZE | SCSI | FLUSH | 
        TOPOLOGY | CONFIG_WCE | DISCARD | WRITE_ZEROES | NOTIFY_ON_EMPTY | 
        RING_INDIRECT_DESC | RING_EVENT_IDX
[ INFO] config: 0x10001100
[ INFO] found a block device of size 65536KB
Usertests: Running 00hello_world
Usertests: Running 05write_a
...
```

- 内核启动时间：< 1 秒
- 块设备识别：成功
- 用户测试执行：立即开始

## 修复的文件

1. **test-qemu.sh** - 简化的 QEMU 启动脚本
   - 添加 `-bios none` 禁用 OpenSBI
   - 使用正确的 virtio-mmio 总线配置

2. **run-in-qemu-system.sh** - 高级启动脚本（5 处修改）
   - interactive 模式
   - ch18_file0-3 自动化测试模式

## 使用方法

```bash
cd /home/chyyuu/thecodes/os-compare/tg-ch18/linux-compatible-tests

# 方式 1：一键启动（推荐）
./quickstart.sh

# 方式 2：手动控制
./test-qemu.sh                      # 简单启动
./run-in-qemu-system.sh interactive # 交互式
./run-in-qemu-system.sh ch18_file0  # 运行特定测试
```

## 在 QEMU 中运行测试

启动后可以在提示符中运行：
```bash
ls              # 查看文件
./ch18_file0    # 运行测试
./ch18_file1
./ch18_file2
./ch18_file3
```

## 重要的 QEMU 参数说明

| 参数 | 说明 | 重要性 |
|------|------|--------|
| `-machine virt` | 使用虚拟机器类型 | 必需 |
| `-bios none` | 禁用固件，允许内核直接启动 | **关键** |
| `-drive file=...,if=none,id=x0` | 定义块驱动器 | **关键** |
| `-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0` | 连接到 MMio 总线 | **关键** |
| `-kernel` | 指定内核文件 | 必需 |
| `-nographic` | 禁用图形，使用仅终端 | 推荐 |
| `-m 64M` | 内存大小 | 推荐 |

## 故障排除

### 仍然出现地址冲突？
- 确保使用了 `-bios none`
- 检查 QEMU 版本（建议 >= 9.0）

### Virtio 设备仍然失败？
- 确保 `-device` 中包含 `bus=virtio-mmio-bus.0`
- 检查 `-drive` 使用 `if=none` 而不是 `if=virtio`

### 文件系统镜像不存在？
- 运行 `./pack-to-fsimg.sh` 创建 fs.img
- 验证 `tg-ch18/target/riscv64gc-unknown-none-elf/debug/fs.img` 存在

## 参考资料

- [tg-ch18 官方配置](../../../.cargo/config.toml)
- [QEMU virt 机器文档](https://www.qemu.org/docs/master/system/riscv/virt.html)
- [Virtio 块设备规范](https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html)

---

**状态**：✅ 已完全修复和验证  
**测试日期**：2026-02-05  
**测试环境**：Linux RISC-V64 + QEMU 10.2.0
