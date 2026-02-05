# QEMU 配置 - 快速参考

## ✅ 已修复的问题

| 问题 | 错误信息 | 解决方案 |
|------|----------|---------|
| ROM 地址冲突 | `overlapping ROM regions` | 添加 `-bios none` |
| Virtio 设备失败 | `ZeroDeviceId` | 使用 `bus=virtio-mmio-bus.0` |

## 正确的 QEMU 配置

```bash
qemu-system-riscv64 \
    -machine virt \
    -m 64M \
    -bios none \                                    # 禁用 OpenSBI
    -drive file="$FS_IMG",if=none,format=raw,id=x0 \  # 驱动定义
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \  # 正确的总线
    -kernel "$KERNEL" \
    -nographic
```

## 一键启动

```bash
cd /path/to/linux-compatible-tests
./quickstart.sh
```

这会自动：
1. ✅ 编译 ch18_file* 为 RISC-V64
2. ✅ 打包到 tg-ch18 文件系统
3. ✅ 启动 QEMU 核心：

## 常见问题排查

### 问题：仍然出现 "overlapping ROM"
**检查清单**：
- [ ] 确认 `-bios none` 在参数中
- [ ] QEMU 版本 >= 9.0
- [ ] 检查 `test-qemu.sh` 是否使用了最新版本

### 问题：`ZeroDeviceId` 错误
**检查清单**：
- [ ] `-drive` 参数中有 `if=none,id=x0`
- [ ] `-device` 参数中有 `bus=virtio-mmio-bus.0`
- [ ] QEMU 版本兼容

### 问题：fs.img 不存在
**解决**：
```bash
./pack-to-fsimg.sh
```

## 验证步骤

1. **检查文件存在**
   ```bash
   ls -lh tg-ch18/target/riscv64gc-unknown-none-elf/debug/{tg-ch18,fs.img}
   ```

2. **检查脚本配置**
   ```bash
   grep "bios none" test-qemu.sh
   grep "virtio-mmio-bus" test-qemu.sh
   ```

3. **测试启动**
   ```bash
   timeout 5 ./test-qemu.sh 2>&1 | grep -E "found a block device|panicked"
   ```

## 关键参数映射

| QEMU 参数 | 作用 | 替代方案 |
|-----------|------|---------|
| `-bios none` | 禁用固件 | 无 |
| `if=none` | 未附加驱动器 | 不能用 `if=virtio` |
| `id=x0` | 驱动器标识符 | 可以改名 |
| `bus=virtio-mmio-bus.0` | 指定总线 | 必需 |

## 性能指标

| 阶段 | 时间 |
|------|------|
| QEMU 启动 | < 1 秒 |
| 内核初始化 | < 1 秒 |
| 块设备识别 | ~ 100 ms |
| 用户程序启动 | 立即 |

**总启动时间**：~ 2-3 秒

---

**最后更新**：2026-02-05 ✅
