# 数据段：存放要输出的字符串
.data
    msg: .string "hello\n"       # 要输出的字符串，包含换行符
    len = . - msg                # 计算字符串长度（当前地址 - 字符串起始地址）

# 代码段：程序执行逻辑
.text
.global _start                   # 定义程序入口点

_start:
    # 第一步：调用write系统调用输出字符串
    li a0, 1                     # a0 = 1 (标准输出stdout的文件描述符)
    la a1, msg                   # a1 = 字符串的内存地址
    li a2, len                   # a2 = 字符串长度
    li a7, 64                    # a7 = 64 (Linux RISC-V64的write系统调用号)
    ecall                        # 触发系统调用

    # 第二步：调用exit系统调用正常退出
    li a0, 0                     # a0 = 0 (退出码，0表示正常退出)
    li a7, 93                    # a7 = 93 (Linux RISC-V64的exit系统调用号)
    ecall                        # 触发系统调用
