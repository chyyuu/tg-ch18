//! 第八章：并发
//!
//! 本章实现了线程和同步原语，支持多线程编程和同步机制。
#![no_std]
#![no_main]
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code, unused_imports))]

mod fs;
mod process;
mod processor;
mod virtio_block;

#[macro_use]
extern crate tg_console;

#[macro_use]
extern crate alloc;

use crate::{
    fs::{read_all, FS},
    impls::{Sv39Manager, SyscallContext},
    process::Process,
    processor::{ProcManager, ProcessorInner, ThreadManager},
};
use alloc::alloc::alloc;
use core::{alloc::Layout, cell::UnsafeCell, mem::MaybeUninit};
use impls::Console;
pub use processor::PROCESSOR;
use riscv::register::*;
#[cfg(not(target_arch = "riscv64"))]
use stub::Sv39;
use tg_console::log;
use tg_easy_fs::{FSManager, OpenFlags};
use tg_kernel_context::foreign::MultislotPortal;
#[cfg(target_arch = "riscv64")]
use tg_kernel_vm::page_table::Sv39;
use tg_kernel_vm::{
    page_table::{MmuMeta, VAddr, VmFlags, VmMeta, PPN, VPN},
    AddressSpace,
};
use tg_sbi;
use tg_signal::SignalResult;
use tg_syscall::Caller;
use tg_task_manage::ProcId;
use xmas_elf::ElfFile;

/// 构建 VmFlags。
#[cfg(target_arch = "riscv64")]
const fn build_flags(s: &str) -> VmFlags<Sv39> {
    VmFlags::build_from_str(s)
}

/// 解析 VmFlags。
#[cfg(target_arch = "riscv64")]
fn parse_flags(s: &str) -> Result<VmFlags<Sv39>, ()> {
    s.parse()
}

#[cfg(not(target_arch = "riscv64"))]
use stub::{build_flags, parse_flags};

// 定义内核入口。
#[cfg(target_arch = "riscv64")]
tg_linker::boot0!(rust_main; stack = 32 * 4096);
// 物理内存容量 = 48 MiB。
const MEMORY: usize = 48 << 20;
// 传送门所在虚页。
const PROTAL_TRANSIT: VPN<Sv39> = VPN::MAX;
struct KernelSpace {
    inner: UnsafeCell<MaybeUninit<AddressSpace<Sv39, Sv39Manager>>>,
}

unsafe impl Sync for KernelSpace {}

impl KernelSpace {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    unsafe fn write(&self, space: AddressSpace<Sv39, Sv39Manager>) {
        *self.inner.get() = MaybeUninit::new(space);
    }

    unsafe fn assume_init_ref(&self) -> &AddressSpace<Sv39, Sv39Manager> {
        &*(*self.inner.get()).as_ptr()
    }
}

// 内核地址空间。
static KERNEL_SPACE: KernelSpace = KernelSpace::new();

extern "C" fn rust_main() -> ! {
    let layout = tg_linker::KernelLayout::locate();
    // bss 段清零
    unsafe { layout.zero_bss() };
    // 初始化 `console`
    tg_console::init_console(&Console);
    tg_console::set_log_level(option_env!("LOG"));
    tg_console::test_log();
    // 初始化内核堆
    tg_kernel_alloc::init(layout.start() as _);
    unsafe {
        tg_kernel_alloc::transfer(core::slice::from_raw_parts_mut(
            layout.end() as _,
            MEMORY - layout.len(),
        ))
    };
    // 建立异界传送门
    let portal_size = MultislotPortal::calculate_size(1);
    let portal_layout = Layout::from_size_align(portal_size, 1 << Sv39::PAGE_BITS).unwrap();
    let portal_ptr = unsafe { alloc(portal_layout) };
    assert!(portal_layout.size() < 1 << Sv39::PAGE_BITS);
    // 建立内核地址空间
    kernel_space(layout, MEMORY, portal_ptr as _);
    // 初始化异界传送门
    let portal = unsafe { MultislotPortal::init_transit(PROTAL_TRANSIT.base().val(), 1) };
    // 初始化 syscall
    tg_syscall::init_io(&SyscallContext);
    tg_syscall::init_process(&SyscallContext);
    tg_syscall::init_scheduling(&SyscallContext);
    tg_syscall::init_clock(&SyscallContext);
    tg_syscall::init_signal(&SyscallContext);
    tg_syscall::init_memory(&SyscallContext);
    let initproc = read_all(FS.open("initproc", OpenFlags::RDONLY).unwrap());
    if let Some((process, thread)) = Process::from_elf(ElfFile::new(initproc.as_slice()).unwrap()) {
        PROCESSOR.get_mut().set_proc_manager(ProcManager::new());
        PROCESSOR.get_mut().set_manager(ThreadManager::new());
        let (pid, tid) = (process.pid, thread.tid);
        PROCESSOR
            .get_mut()
            .add_proc(pid, process, ProcId::from_usize(usize::MAX));
        PROCESSOR.get_mut().add(tid, thread, pid);
    }
    loop {
        let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
        if let Some(task) = unsafe { (*processor).find_next() } {
            unsafe { task.context.execute(portal, ()) };
            match scause::read().cause() {
                scause::Trap::Exception(scause::Exception::UserEnvCall) => {
                    use tg_syscall::{SyscallId as Id, SyscallResult as Ret};
                    let ctx = &mut task.context.context;
                    ctx.move_next();
                    let id: Id = ctx.a(7).into();
                    let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];
                    let syscall_ret = tg_syscall::handle(Caller { entity: 0, flow: 0 }, id, args);
                    // 目前信号处理位置放在 syscall 执行之后，这只是临时的实现。
                    // 正确处理信号的位置应该是在 “trap 中处理异常和中断和异常之后，返回用户态之前”。
                    // 例如发现有访存异常时，应该触发 SIGSEGV 信号然后进行处理。
                    // 但目前 syscall 之后直接切换用户程序，没有 “返回用户态” 这一步，甚至 trap 本身也没了。
                    //
                    // 最简单粗暴的方法是，在 `scause::Trap` 分类的每一条分支之后都加上信号处理，
                    // 当然这样可能代码上不够优雅。处理信号的具体时机还需要后续再讨论。
                    let current_proc = unsafe { (*processor).get_current_proc().unwrap() };
                    match current_proc.signal.handle_signals(ctx) {
                        // 进程应该结束执行
                        SignalResult::ProcessKilled(exit_code) => unsafe {
                            (*processor).make_current_exited(exit_code as _)
                        },
                        _ => match syscall_ret {
                            Ret::Done(ret) => match id {
                                Id::EXIT | Id::EXIT_GROUP => unsafe { (*processor).make_current_exited(ret) },
                                _ => {
                                    let ctx = &mut task.context.context;
                                    *ctx.a_mut(0) = ret as _;
                                    unsafe { (*processor).make_current_suspend() };
                                }
                            },
                            Ret::Unsupported(_) => {
                                log::error!("Unsupported syscall: id = {id:?}");
                                log::error!("  Syscall args: [{:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}]", 
                                    args[0], args[1], args[2], args[3], args[4], args[5]);
                                log::error!("  Process will exit with code -2");
                                unsafe { (*processor).make_current_exited(-2) };
                            }
                        },
                    }
                }
                scause::Trap::Exception(scause::Exception::Breakpoint) => {
                    // 处理 breakpoint 异常 (ebreak 指令)
                    // breakpoint 在 glibc 中通常表示到达了某个检查点
                    // 简化处理：程序正常退出，表示已完成
                    let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
                    let sepc_val = sepc::read();
                    log::info!("Program reached breakpoint at {:#x}, exiting with success", sepc_val);
                    unsafe { (*processor).make_current_exited(0) };
                }
                e => {
                    let ctx = &task.context.context;
                    let current_proc = unsafe { (*processor).get_current_proc() };
                    let pid = current_proc.map(|p| p.pid.get_usize()).unwrap_or(0);
                    
                    let sepc_val = sepc::read();
                    let stval_val = stval::read();
                    
                    log::error!("╔════════════════════════════════════════════════════════════╗");
                    log::error!("║  Unsupported Trap Exception                                 ║");
                    log::error!("╚════════════════════════════════════════════════════════════╝");
                    log::error!("  Trap Type: {e:?}");
                    log::error!("  Process PID: {}", pid);
                    log::error!("  Exception PC (sepc): {:#x}", sepc_val);
                    log::error!("  Exception Value (stval): {:#x}", stval_val);
                    log::error!("  Register a0: {:#x}", ctx.a(0));
                    log::error!("  Register a1: {:#x}", ctx.a(1));
                    log::error!("  Register a5: {:#x}", ctx.a(5));
                    log::error!("  Register a7 (syscall id): {:#x}", ctx.a(7));
                    log::error!("  Register sp: {:#x}", ctx.sp());
                    log::error!("  Register ra: {:#x}", ctx.ra());
                    log::error!("  Process will exit with code -3");
                    log::error!("════════════════════════════════════════════════════════════");
                    
                    unsafe { (*processor).make_current_exited(-3) };
                }
            }
        } else {
            println!("no task");
            break;
        }
    }

    tg_sbi::shutdown(false)
}

/// Rust 异常处理函数，以异常方式关机。
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    tg_sbi::shutdown(true)
}

/// Virtio Block in virt machine
pub const MMIO: &[(usize, usize)] = &[(0x1000_1000, 0x00_1000)];

fn kernel_space(layout: tg_linker::KernelLayout, memory: usize, portal: usize) {
    let mut space = AddressSpace::new();
    for region in layout.iter() {
        log::info!("{region}");
        use tg_linker::KernelRegionTitle::*;
        let flags = match region.title {
            Text => "X_RV",
            Rodata => "__RV",
            Data | Boot => "_WRV",
        };
        let s = VAddr::<Sv39>::new(region.range.start);
        let e = VAddr::<Sv39>::new(region.range.end);
        space.map_extern(
            s.floor()..e.ceil(),
            PPN::new(s.floor().val()),
            build_flags(flags),
        )
    }
    let s = VAddr::<Sv39>::new(layout.end());
    let e = VAddr::<Sv39>::new(layout.start() + memory);
    log::info!("(heap) ---> {:#10x}..{:#10x}", s.val(), e.val());
    space.map_extern(
        s.floor()..e.ceil(),
        PPN::new(s.floor().val()),
        build_flags("_WRV"),
    );
    space.map_extern(
        PROTAL_TRANSIT..PROTAL_TRANSIT + 1,
        PPN::new(portal >> Sv39::PAGE_BITS),
        build_flags("__G_XWRV"),
    );
    println!();

    // MMIO
    for (base, len) in MMIO {
        let s = VAddr::<Sv39>::new(*base);
        let e = VAddr::<Sv39>::new(*base + *len);
        log::info!("MMIO range -> {:#10x}..{:#10x}", s.val(), e.val());
        space.map_extern(
            s.floor()..e.ceil(),
            PPN::new(s.floor().val()),
            build_flags("_WRV"),
        );
    }

    unsafe { satp::set(satp::Mode::Sv39, 0, space.root_ppn().val()) };
    unsafe { KERNEL_SPACE.write(space) };
}

/// 映射异界传送门。
fn map_portal(space: &AddressSpace<Sv39, Sv39Manager>) {
    let portal_idx = PROTAL_TRANSIT.index_in(Sv39::MAX_LEVEL);
    space.root()[portal_idx] = unsafe { KERNEL_SPACE.assume_init_ref() }.root()[portal_idx];
}

/// 各种接口库的实现。
mod impls {
    use crate::{
        build_flags,
        fs::{read_all, Fd, FS},
        processor::ProcessorInner,
        Sv39, PROCESSOR,
    };
    use alloc::{alloc::alloc_zeroed, string::String, vec::Vec};
    use core::{alloc::Layout, ptr::NonNull};
    use spin::Mutex;
    use tg_console::log;
    use tg_easy_fs::{make_pipe, FSManager, OpenFlags, UserBuffer};
    use tg_kernel_vm::{
        page_table::{MmuMeta, Pte, VAddr, VmFlags, PPN, VPN},
        PageManager,
    };
    use tg_signal::SignalNo;
    use tg_syscall::*;
    use tg_task_manage::ProcId;
    use xmas_elf::ElfFile;

    #[repr(transparent)]
    pub struct Sv39Manager(NonNull<Pte<Sv39>>);

    impl Sv39Manager {
        const OWNED: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1 << 8) };

        #[inline]
        fn page_alloc<T>(count: usize) -> *mut T {
            unsafe {
                alloc_zeroed(Layout::from_size_align_unchecked(
                    count << Sv39::PAGE_BITS,
                    1 << Sv39::PAGE_BITS,
                ))
            }
            .cast()
        }
    }

    impl PageManager<Sv39> for Sv39Manager {
        #[inline]
        fn new_root() -> Self {
            Self(NonNull::new(Self::page_alloc(1)).unwrap())
        }

        #[inline]
        fn root_ppn(&self) -> PPN<Sv39> {
            PPN::new(self.0.as_ptr() as usize >> Sv39::PAGE_BITS)
        }

        #[inline]
        fn root_ptr(&self) -> NonNull<Pte<Sv39>> {
            self.0
        }

        #[inline]
        fn p_to_v<T>(&self, ppn: PPN<Sv39>) -> NonNull<T> {
            unsafe { NonNull::new_unchecked(VPN::<Sv39>::new(ppn.val()).base().as_mut_ptr()) }
        }

        #[inline]
        fn v_to_p<T>(&self, ptr: NonNull<T>) -> PPN<Sv39> {
            PPN::new(VAddr::<Sv39>::new(ptr.as_ptr() as _).floor().val())
        }

        #[inline]
        fn check_owned(&self, pte: Pte<Sv39>) -> bool {
            pte.flags().contains(Self::OWNED)
        }

        #[inline]
        fn allocate(&mut self, len: usize, flags: &mut VmFlags<Sv39>) -> NonNull<u8> {
            *flags |= Self::OWNED;
            NonNull::new(Self::page_alloc(len)).unwrap()
        }

        fn deallocate(&mut self, _pte: Pte<Sv39>, _len: usize) -> usize {
            todo!()
        }

        fn drop_root(&mut self) {
            todo!()
        }
    }

    pub struct Console;

    impl tg_console::Console for Console {
        #[inline]
        fn put_char(&self, c: u8) {
            tg_sbi::console_putchar(c);
        }
    }

    pub struct SyscallContext;
    const READABLE: VmFlags<Sv39> = build_flags("RV");
    const WRITEABLE: VmFlags<Sv39> = build_flags("W_V");

    fn linux_open_flags(flags: u32) -> Option<OpenFlags> {
        const O_ACCMODE: u32 = 0b11;
        const O_WRONLY: u32 = 1;
        const O_RDWR: u32 = 2;
        const O_CREAT: u32 = 0x40;
        const O_TRUNC: u32 = 0x200;

        let mut out = OpenFlags::empty();
        match flags & O_ACCMODE {
            0 => out |= OpenFlags::RDONLY,
            O_WRONLY => out |= OpenFlags::WRONLY,
            O_RDWR => out |= OpenFlags::RDWR,
            _ => return None,
        }
        if flags & O_CREAT != 0 {
            out |= OpenFlags::CREATE;
        }
        if flags & O_TRUNC != 0 {
            out |= OpenFlags::TRUNC;
        }
        Some(out)
    }

    impl IO for SyscallContext {
        fn write(&self, _caller: Caller, fd: usize, buf: usize, count: usize) -> isize {
            log::debug!("sys_write <= fd: {}, buf: {:#x}, count: {}", fd, buf, count);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if let Some(ptr) = current.address_space.translate(VAddr::new(buf), READABLE) {
                if fd == STDOUT || fd == STDDEBUG {
                    print!("{}", unsafe {
                        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            ptr.as_ptr(),
                            count,
                        ))
                    });
                    count as _
                } else if let Some(file) = &current.fd_table[fd] {
                    let file = file.lock();
                    if file.writable() {
                        let mut v: Vec<&'static mut [u8]> = Vec::new();
                        unsafe { v.push(core::slice::from_raw_parts_mut(ptr.as_ptr(), count)) };
                        file.write(UserBuffer::new(v)) as _
                    } else {
                        log::error!("file not writable");
                        -1
                    }
                } else {
                    log::error!("unsupported fd: {fd}");
                    -1
                }
            } else {
                log::error!("ptr not readable");
                -1
            }
        }

        fn read(&self, _caller: Caller, fd: usize, buf: usize, count: usize) -> isize {
            log::debug!("sys_read <= fd: {}, buf: {:#x}, count: {}", fd, buf, count);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if let Some(ptr) = current.address_space.translate(VAddr::new(buf), WRITEABLE) {
                if fd == STDIN {
                    let mut ptr = ptr.as_ptr();
                    for _ in 0..count {
                        unsafe {
                            *ptr = tg_sbi::console_getchar() as u8;
                            ptr = ptr.add(1);
                        }
                    }
                    count as _
                } else if let Some(file) = &current.fd_table[fd] {
                    let file = file.lock();
                    if file.readable() {
                        let mut v: Vec<&'static mut [u8]> = Vec::new();
                        unsafe { v.push(core::slice::from_raw_parts_mut(ptr.as_ptr(), count)) };
                        file.read(UserBuffer::new(v)) as _
                    } else {
                        log::error!("file not readable");
                        -1
                    }
                } else {
                    log::error!("unsupported fd: {fd}");
                    -1
                }
            } else {
                log::error!("ptr not writeable");
                -1
            }
        }

        fn open(&self, _caller: Caller, dirfd: isize, path: usize, flags: usize, _mode: usize) -> isize {
            log::debug!("sys_openat <= dirfd: {}, path: {:#x}, flags: {:#x}, mode: {:#x}", dirfd, path, flags, _mode);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if let Some(ptr) = current.address_space.translate(VAddr::new(path), READABLE) {
                let mut string = String::new();
                let mut raw_ptr: *mut u8 = ptr.as_ptr();
                loop {
                    unsafe {
                        let ch = *raw_ptr;
                        if ch == 0 {
                            break;
                        }
                        string.push(ch as char);
                        raw_ptr = (raw_ptr as usize + 1) as *mut u8;
                    }
                }

                let flags = match linux_open_flags(flags as u32) {
                    Some(flags) => flags,
                    None => return -1,
                };
                if let Some(file_handle) = FS.open(string.as_str(), flags) {
                    let new_fd = current.fd_table.len();
                    // Arc<FileHandle> -> FileHandle，需要解引用
                    current
                        .fd_table
                        .push(Some(Mutex::new(Fd::File((*file_handle).clone()))));
                    new_fd as isize
                } else {
                    -1
                }
            } else {
                log::error!("ptr not writeable");
                -1
            }
        }

        fn fstat(&self, _caller: Caller, fd: usize, st: usize) -> isize {
            log::debug!("sys_fstat <= fd: {}, st: {:#x}", fd, st);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if fd >= current.fd_table.len() || current.fd_table[fd].is_none() {
                return -1;
            }
            if let Some(mut ptr) = current
                .address_space
                .translate::<Stat>(VAddr::new(st), WRITEABLE)
            {
                let stat = unsafe { ptr.as_mut() };
                let file = current.fd_table[fd].as_ref().unwrap().lock();
                match &*file {
                    Fd::File(f) => {
                        if let Some(inode) = &f.inode {
                            let inode_id = inode.inode_id();
                            let is_dir = inode.is_dir();
                            let mode = if is_dir {
                                StatMode::S_IFDIR | StatMode::DEFAULT_DIR_PERM
                            } else {
                                StatMode::S_IFREG | StatMode::DEFAULT_FILE_PERM
                            };
                            *stat = Stat {
                                st_dev: 0,
                                st_ino: inode_id as u64,
                                st_mode: mode,
                                st_nlink: FS.count_links(inode_id),
                                st_size: inode.size() as i64,
                            };
                            0
                        } else {
                            -1
                        }
                    }
                    _ => -1,
                }
            } else {
                log::error!("ptr not writeable");
                -1
            }
        }

        #[inline]
        fn close(&self, _caller: Caller, fd: usize) -> isize {
            log::debug!("sys_close <= fd: {}", fd);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if fd >= current.fd_table.len() || current.fd_table[fd].is_none() {
                return -1;
            }
            current.fd_table[fd].take();
            0
        }

        fn pipe(&self, _caller: Caller, pipe: usize) -> isize {
            log::debug!("sys_pipe <= pipe: {:#x}", pipe);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            let (read_end, write_end) = make_pipe();
            let read_fd = current.fd_table.len();
            let write_fd = read_fd + 1;
            // 将 read_fd 写入 pipe[0]
            if let Some(mut ptr) = current
                .address_space
                .translate::<usize>(VAddr::new(pipe), WRITEABLE)
            {
                unsafe { *ptr.as_mut() = read_fd };
            } else {
                return -1;
            }
            // 将 write_fd 写入 pipe[1]
            if let Some(mut ptr) = current
                .address_space
                .translate::<usize>(VAddr::new(pipe + core::mem::size_of::<usize>()), WRITEABLE)
            {
                unsafe { *ptr.as_mut() = write_fd };
            } else {
                return -1;
            }
            // 最后添加，避免中途写入异常导致浪费一个 fd
            current
                .fd_table
                .push(Some(Mutex::new(Fd::PipeRead(read_end))));
            current
                .fd_table
                .push(Some(Mutex::new(Fd::PipeWrite(write_end))));
            0
        }
        
        fn readlinkat(&self, _caller: Caller, _dirfd: i32, path: usize, buf: usize, bufsize: usize) -> isize {
            log::debug!("sys_readlinkat <= dirfd: {}, path: {:#x}, buf: {:#x}, bufsize: {}", _dirfd, path, buf, bufsize);
            // 简化实现：不支持符号链接，返回 EINVAL
            // 完整实现需要：
            // 1. 解析 path 字符串
            // 2. 检查文件是否是符号链接
            // 3. 读取链接目标并写入 buf
            let _ = (path, buf, bufsize);
            -22 // -EINVAL，表示参数无效或不是符号链接
        }
        
        fn dup(&self, _caller: Caller, oldfd: usize) -> isize {
            log::debug!("sys_dup <= oldfd: {}", oldfd);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            
            // 检查 oldfd 是否有效
            if oldfd >= current.fd_table.len() || current.fd_table[oldfd].is_none() {
                return -9; // -EBADF: Bad file descriptor
            }
            
            // 获取旧的文件描述符对应的文件对象，并克隆
            let new_file = {
                let old_file = current.fd_table[oldfd].as_ref().unwrap();
                old_file.lock().clone()
            };
            
            // 现在可以安全地对 fd_table 进行可变操作
            let new_fd = current.fd_table.len();
            current.fd_table.push(Some(Mutex::new(new_file)));
            
            new_fd as isize
        }
        
        fn fcntl(&self, _caller: Caller, fd: usize, cmd: i32, _arg: usize) -> isize {
            log::debug!("sys_fcntl <= fd: {}, cmd: {}, arg: {}", fd, cmd, _arg);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            
            // 检查文件描述符是否有效
            if fd >= current.fd_table.len() || current.fd_table[fd].is_none() {
                return -9; // -EBADF
            }
            
            // fcntl 命令常数
            const F_DUPFD: i32 = 0;      // 复制文件描述符
            const F_GETFD: i32 = 1;      // 获取关闭时执行标志
            const F_SETFD: i32 = 2;      // 设置关闭时执行标志
            const F_GETFL: i32 = 3;      // 获取文件打开标志
            const F_SETFL: i32 = 4;      // 设置文件打开标志
            const O_RDWR: i32 = 2;       // 读写打开标志
            
            match cmd {
                F_DUPFD => {
                    // F_DUPFD: 复制 fd，使用 >= _arg 的最小可用文件描述符
                    // 简化实现：忽略 _arg 参数，直接复制到最后
                    let new_file = {
                        let old_file = current.fd_table[fd].as_ref().unwrap();
                        old_file.lock().clone()
                    };
                    let new_fd = current.fd_table.len();
                    current.fd_table.push(Some(Mutex::new(new_file)));
                    new_fd as isize
                }
                F_GETFD => {
                    // 获取 close-on-exec 标志，简化实现返回 0
                    0
                }
                F_SETFD => {
                    // 设置 close-on-exec 标志，简化实现返回 0
                    0
                }
                F_GETFL => {
                    // 获取打开标志
                    // 简化实现：对于所有文件返回 O_RDWR
                    O_RDWR as isize
                }
                F_SETFL => {
                    // 设置打开标志（只有 O_NONBLOCK 等被允许改变），简化实现返回 0
                    0
                }
                _ => {
                    // 未支持的 fcntl 命令，返回 -EINVAL
                    -22
                }
            }
        }
    }

    impl Process for SyscallContext {
        #[inline]
        fn exit(&self, _caller: Caller, exit_code: usize) -> isize {
            log::debug!("sys_exit <= exit_code: {}", exit_code);
            exit_code as isize
        }

        fn exit_group(&self, _caller: Caller, exit_code: usize) -> isize {
            log::debug!("sys_exit_group <= exit_code: {}", exit_code);
            // exit_group 与 exit 有相同的行为：退出整个进程
            exit_code as isize
        }

        fn fork(&self, _caller: Caller) -> isize {
            log::debug!("sys_fork <=");
            let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
            let current_proc = unsafe { (*processor).get_current_proc().unwrap() };
            let parent_pid = current_proc.pid; // 先保存父进程 pid
            let (proc, mut thread) = current_proc.fork().unwrap();
            let pid = proc.pid;
            *thread.context.context.a_mut(0) = 0 as _;
            unsafe {
                (*processor).add_proc(pid, proc, parent_pid);
                (*processor).add(thread.tid, thread, pid);
            }
            pid.get_usize() as isize
        }

        fn exec(&self, _caller: Caller, path: usize, count: usize) -> isize {
            log::debug!("sys_exec <= path: {:#x}, count: {}", path, count);
            const READABLE: VmFlags<Sv39> = build_flags("RV");
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            current
                .address_space
                .translate(VAddr::new(path), READABLE)
                .map(|ptr| unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr.as_ptr(), count))
                })
                .and_then(|name| FS.open(name, OpenFlags::RDONLY))
                .map_or_else(
                    || {
                        log::error!("unknown app, select one in the list: ");
                        FS.readdir("")
                            .unwrap()
                            .into_iter()
                            .for_each(|app| println!("{app}"));
                        println!();
                        -1
                    },
                    |fd| {
                        current.exec(ElfFile::new(&read_all(fd)).unwrap());
                        0
                    },
                )
        }

        fn wait(&self, _caller: Caller, pid: isize, exit_code_ptr: usize) -> isize {
            log::debug!("sys_wait <= pid: {}, exit_code_ptr: {:#x}", pid, exit_code_ptr);
            let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
            let current = unsafe { (*processor).get_current_proc().unwrap() };
            const WRITABLE: VmFlags<Sv39> = build_flags("W_V");
            if let Some((dead_pid, exit_code)) =
                unsafe { (*processor).wait(ProcId::from_usize(pid as usize)) }
            {
                if let Some(mut ptr) = current
                    .address_space
                    .translate::<i32>(VAddr::new(exit_code_ptr), WRITABLE)
                {
                    unsafe { *ptr.as_mut() = exit_code as i32 };
                }
                return dead_pid.get_usize() as isize;
            } else {
                // 等待的子进程不存在
                return -1;
            }
        }

        fn getpid(&self, _caller: Caller) -> isize {
            log::debug!("sys_getpid <=");
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            current.pid.get_usize() as _
        }
        
        fn set_tid_address(&self, _caller: Caller, _tidp: usize) -> isize {
            log::debug!("sys_set_tid_address <= tidp: {:#x}", _tidp);
            // 简化实现：只是返回 PID，不实际使用 _tidp
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            current.pid.get_usize() as isize
        }
        
        fn set_robust_list(&self, _caller: Caller, _head: usize, _len: usize) -> isize {
            log::debug!("sys_set_robust_list <= head: {:#x}, len: {}", _head, _len);
            // 简化实现：futex robust list 对单线程程序不是必需的
            // 返回 0 表示成功
            0
        }
        
        fn prlimit64(&self, _caller: Caller, _pid: isize, resource: u32, new_limit: usize, old_limit: usize) -> isize {
            log::debug!("sys_prlimit64 <= pid: {}, resource: {}, new_limit: {:#x}, old_limit: {:#x}", _pid, resource, new_limit, old_limit);
            use linux_raw_sys::general::{RLIM_NLIMITS, rlimit64};
            
            // 检查资源类型是否有效
            if resource >= RLIM_NLIMITS {
                return -22; // -EINVAL
            }
            
            // 如果需要返回旧的限制
            if old_limit != 0 {
                const WRITABLE: VmFlags<Sv39> = build_flags("W_V");
                let current = PROCESSOR.get_mut().get_current_proc().unwrap();
                if let Some(mut ptr) = current
                    .address_space
                    .translate::<rlimit64>(VAddr::new(old_limit), WRITABLE)
                {
                    unsafe {
                        // 返回一个默认的资源限制（无限制）
                        *ptr.as_mut() = rlimit64 {
                            rlim_cur: u64::MAX,
                            rlim_max: u64::MAX,
                        };
                    }
                }
            }
            
            // 简化实现：忽略新的限制设置（new_limit）
            // 完整实现应该存储这些限制并在资源使用时检查
            let _ = new_limit; // 避免未使用变量警告
            0
        }
    }

    impl Scheduling for SyscallContext {
        #[inline]
        fn sched_yield(&self, _caller: Caller) -> isize {
            log::debug!("sys_sched_yield <=");
            0
        }
        
        fn nanosleep(&self, _caller: Caller, _req: usize, _rem: usize) -> isize {
            log::debug!("sys_nanosleep <= req: {:#x}, rem: {:#x}", _req, _rem);
            // 简化实现：不真正睡眠，直接返回 0（成功）
            // 在真实环境下应该解析 timespec 结构体并让出 CPU
            0
        }
    }

    impl Clock for SyscallContext {
        #[inline]
        fn clock_gettime(&self, _caller: Caller, clock_id: ClockId, tp: usize) -> isize {
            log::debug!("sys_clock_gettime <= clock_id: {:?}, tp: {:#x}", clock_id, tp);
            const WRITABLE: VmFlags<Sv39> = build_flags("W_V");
            match clock_id {
                ClockId::CLOCK_MONOTONIC => {
                    if let Some(mut ptr) = PROCESSOR
                        .get_mut()
                        .get_current_proc()
                        .unwrap()
                        .address_space
                        .translate(VAddr::new(tp), WRITABLE)
                    {
                        let time = riscv::register::time::read() * 10000 / 125;
                        *unsafe { ptr.as_mut() } = TimeSpec {
                            tv_sec: time / 1_000_000_000,
                            tv_nsec: time % 1_000_000_000,
                        };
                        0
                    } else {
                        log::error!("ptr not readable");
                        -1
                    }
                }
                _ => -1,
            }
        }
    }

    impl Signal for SyscallContext {
        fn kill(&self, _caller: Caller, pid: isize, signum: u8) -> isize {
            log::debug!("sys_kill <= pid: {}, signum: {}", pid, signum);
            if let Some(target_task) = PROCESSOR
                .get_mut()
                .get_proc(ProcId::from_usize(pid as usize))
            {
                if let Ok(signal_no) = SignalNo::try_from(signum) {
                    if signal_no != SignalNo::ERR {
                        target_task.signal.add_signal(signal_no);
                        return 0;
                    }
                }
            }
            -1
        }

        fn sigaction(
            &self,
            _caller: Caller,
            signum: u8,
            action: usize,
            old_action: usize,
        ) -> isize {
            log::debug!("sys_sigaction <= signum: {}, action: {:#x}, old_action: {:#x}", signum, action, old_action);
            if signum as usize > tg_signal::MAX_SIG {
                return -1;
            }
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            if let Ok(signal_no) = SignalNo::try_from(signum) {
                if signal_no == SignalNo::ERR {
                    return -1;
                }
                // 如果需要返回原来的处理函数，则从信号模块中获取
                if old_action as usize != 0 {
                    if let Some(mut ptr) = current
                        .address_space
                        .translate(VAddr::new(old_action), WRITEABLE)
                    {
                        if let Some(signal_action) = current.signal.get_action_ref(signal_no) {
                            *unsafe { ptr.as_mut() } = signal_action;
                        } else {
                            return -1;
                        }
                    } else {
                        // 如果返回了 None，说明 signal_no 无效
                        return -1;
                    }
                }
                // 如果需要设置新的处理函数，则设置到信号模块中
                if action as usize != 0 {
                    if let Some(ptr) = current
                        .address_space
                        .translate(VAddr::new(action), READABLE)
                    {
                        // 如果返回了 false，说明 signal_no 无效
                        if !current
                            .signal
                            .set_action(signal_no, &unsafe { *ptr.as_ptr() })
                        {
                            return -1;
                        }
                    } else {
                        return -1;
                    }
                }
                return 0;
            }
            -1
        }

        fn sigprocmask(&self, _caller: Caller, mask: usize) -> isize {
            log::debug!("sys_sigprocmask <= mask: {:#x}", mask);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            current.signal.update_mask(mask) as isize
        }

        fn sigreturn(&self, _caller: Caller) -> isize {
            log::debug!("sys_sigreturn <=");
            let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
            let current = unsafe { (*processor).get_current_proc().unwrap() };
            let current_thread = unsafe { (*processor).current().unwrap() };
            // 如成功，则需要修改当前用户程序的 LocalContext
            if current
                .signal
                .sig_return(&mut current_thread.context.context)
            {
                0
            } else {
                -1
            }
        }

        fn rt_sigpending(&self, _caller: Caller, _set: usize, _sigsetsize: usize) -> isize {
            log::debug!("sys_rt_sigpending <= set: {:#x}, sigsetsize: {}", _set, _sigsetsize);
            // 简化实现：没有待处理的信号
            0
        }
    }

    impl Memory for SyscallContext {
        fn brk(&self, _caller: Caller, addr: usize) -> isize {
            log::debug!("sys_brk <= addr: {:#x}", addr);
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            
            log::info!("brk called: addr={:#x}, current heap_start={:#x}, heap_end={:#x}", 
                       addr, current.heap_start, current.heap_end);
            
            // 如果 addr == 0，返回当前的堆边界
            if addr == 0 {
                log::info!("brk(0) returning heap_end={:#x}", current.heap_end);
                return current.heap_end as isize;
            }
            
            // 确保请求的地址不低于堆起始地址
            if addr < current.heap_start {
                return current.heap_end as isize;  // 失败，返回当前边界
            }
            
            // 确保请求的地址不会与栈冲突（简单检查：不超过某个上限）
            const MAX_HEAP_ADDR: usize = 0x10000000;  // 256MB 堆上限
            if addr > MAX_HEAP_ADDR {
                return current.heap_end as isize;  // 失败，返回当前边界
            }
            
            let old_heap_end = current.heap_end;
            let new_heap_end = addr;
            
            // 如果新地址比旧地址大，需要分配新页
            if new_heap_end > old_heap_end {
                const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
                let old_heap_end_page = (old_heap_end + PAGE_SIZE - 1) / PAGE_SIZE;
                let new_heap_end_page = (new_heap_end + PAGE_SIZE - 1) / PAGE_SIZE;
                
                // 需要映射新页
                if new_heap_end_page > old_heap_end_page {
                    let pages_to_map = new_heap_end_page - old_heap_end_page;
                    let start_vpn = VPN::new(old_heap_end_page);
                    let end_vpn = VPN::new(new_heap_end_page);
                    
                    // 分配物理内存
                    let layout = Layout::from_size_align(
                        pages_to_map * PAGE_SIZE,
                        PAGE_SIZE,
                    ).unwrap();
                    let ptr = unsafe { alloc_zeroed(layout) };
                    
                    // 映射到地址空间
                    current.address_space.map_extern(
                        start_vpn..end_vpn,
                        PPN::new(ptr as usize >> Sv39::PAGE_BITS),
                        build_flags("U_WRV"),
                    );
                }
            }
            // 如果新地址比旧地址小，理论上应该释放页面，但我们简化处理，不实际释放
            
            // 更新堆边界
            current.heap_end = new_heap_end;
            new_heap_end as isize
        }
        
        fn getrandom(&self, _caller: Caller, buf: usize, len: usize, _flags: u32) -> isize {
            log::debug!("sys_getrandom <= buf: {:#x}, len: {}, flags: {:#x}", buf, len, _flags);
            // 简化实现：填充伪随机数
            // 完整实现应该使用真随机数生成器（如 /dev/urandom）
            if len == 0 {
                return 0;
            }
            
            const WRITABLE: VmFlags<Sv39> = build_flags("W_V");
            let current = PROCESSOR.get_mut().get_current_proc().unwrap();
            
            // 简单的伪随机数生成（使用时间戳作为种子）
            static mut SEED: usize = 0x123456789abcdef;
            
            let mut written = 0;
            for i in 0..len {
                if let Some(mut ptr) = current
                    .address_space
                    .translate::<u8>(VAddr::new(buf + i), WRITABLE)
                {
                    unsafe {
                        // 简单的线性同余生成器
                        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
                        *ptr.as_mut() = (SEED >> 16) as u8;
                    }
                    written += 1;
                } else {
                    // 地址无效
                    return if written > 0 { written } else { -14 }; // -EFAULT
                }
            }
            
            written
        }
        
        fn mprotect(&self, _caller: Caller, addr: usize, len: usize, prot: i32) -> isize {
            log::debug!("sys_mprotect <= addr: {:#x}, len: {:#x}, prot: {}", addr, len, prot);
            // 简化实现：只检查参数有效性，不实际修改页表属性
            // 完整实现需要修改页表条目的权限位
            
            // 检查地址是否对齐
            const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
            if addr % PAGE_SIZE != 0 {
                return -22; // -EINVAL
            }
            
            // 检查保护标志是否有效（0-7 是有效的组合）
            if prot < 0 || prot > 7 {
                return -22; // -EINVAL
            }
            
            // 检查区域是否有效（简化：不真正检查）
            let _ = len;
            
            // 简化实现：直接返回成功
            // 实际应该修改地址空间中对应页的权限
            0
        }

        fn mmap(
            &self,
            _caller: Caller,
            _addr: usize,
            _length: usize,
            _prot: i32,
            _flags: i32,
            _fd: i32,
            _offset: usize,
        ) -> isize {
            log::debug!("sys_mmap <= addr: {:#x}, length: {:#x}, prot: {}, flags: {}, fd: {}, offset: {:#x}", 
                       _addr, _length, _prot, _flags, _fd, _offset);
            // 暂不实现，返回错误
            -1
        }

        fn munmap(&self, _caller: Caller, _addr: usize, _length: usize) -> isize {
            log::debug!("sys_munmap <= addr: {:#x}, length: {:#x}", _addr, _length);
            // 暂不实现，返回错误
            -1
        }
    }

}

/// 非 RISC-V64 架构的占位实现
#[cfg(not(target_arch = "riscv64"))]
mod stub {
    use tg_kernel_vm::page_table::{MmuMeta, VmFlags};

    /// Sv39 占位类型
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct Sv39;

    impl MmuMeta for Sv39 {
        const P_ADDR_BITS: usize = 56;
        const PAGE_BITS: usize = 12;
        const LEVEL_BITS: &'static [usize] = &[9, 9, 9];
        const PPN_POS: usize = 10;

        #[inline]
        fn is_leaf(value: usize) -> bool {
            value & 0b1110 != 0
        }
    }

    /// 构建 VmFlags 占位。
    pub const fn build_flags(_s: &str) -> VmFlags<Sv39> {
        unsafe { VmFlags::from_raw(0) }
    }

    /// 解析 VmFlags 占位。
    pub fn parse_flags(_s: &str) -> Result<VmFlags<Sv39>, ()> {
        Ok(unsafe { VmFlags::from_raw(0) })
    }

    #[no_mangle]
    pub extern "C" fn main() -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn __libc_start_main() -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn rust_eh_personality() {}
}
