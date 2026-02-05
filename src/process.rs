use crate::{
    build_flags, fs::Fd, map_portal, parse_flags, processor::ProcessorInner, Sv39, Sv39Manager,
    PROCESSOR,
};
use alloc::{alloc::alloc_zeroed, boxed::Box, vec::Vec};
use core::alloc::Layout;
use spin::Mutex;
use tg_kernel_context::{foreign::ForeignContext, LocalContext};
use tg_kernel_vm::{
    page_table::{MmuMeta, VAddr, PPN, VPN},
    AddressSpace,
};
use tg_signal::Signal;
use tg_signal_impl::SignalImpl;
use tg_task_manage::{ProcId, ThreadId};
use xmas_elf::{
    header::{self, HeaderPt2, Machine},
    program, ElfFile,
};

/// 线程
pub struct Thread {
    /// 不可变
    pub tid: ThreadId,
    /// 可变
    pub context: ForeignContext,
}

impl Thread {
    pub fn new(satp: usize, context: LocalContext) -> Self {
        Self {
            tid: ThreadId::new(),
            context: ForeignContext { context, satp },
        }
    }
}

/// 进程。
pub struct Process {
    /// 不可变
    pub pid: ProcId,
    /// 可变
    pub address_space: AddressSpace<Sv39, Sv39Manager>,
    /// 文件描述符表
    pub fd_table: Vec<Option<Mutex<Fd>>>,
    /// 信号模块
    pub signal: Box<dyn Signal>,
    /// 程序堆边界（program break）
    pub heap_start: usize,
    pub heap_end: usize,
}

impl Process {
    /// 只支持一个线程
    pub fn exec(&mut self, elf: ElfFile) {
        let (proc, thread) = Process::from_elf(elf).unwrap();
        self.address_space = proc.address_space;
        self.heap_start = proc.heap_start;
        self.heap_end = proc.heap_end;
        let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
        unsafe {
            let pthreads = (*processor).get_thread(self.pid).unwrap();
            (*processor).get_task(pthreads[0]).unwrap().context = thread.context;
        }
    }
    /// 只支持一个线程
    pub fn fork(&mut self) -> Option<(Self, Thread)> {
        // 子进程 pid
        let pid = ProcId::new();
        // 复制父进程地址空间
        let parent_addr_space = &self.address_space;
        let mut address_space: AddressSpace<Sv39, Sv39Manager> = AddressSpace::new();
        parent_addr_space.cloneself(&mut address_space);
        map_portal(&address_space);
        // 线程
        let processor: *mut ProcessorInner = PROCESSOR.get_mut() as *mut ProcessorInner;
        let pthreads = unsafe { (*processor).get_thread(self.pid).unwrap() };
        let context = unsafe {
            (*processor)
                .get_task(pthreads[0])
                .unwrap()
                .context
                .context
                .clone()
        };
        let satp = (8 << 60) | address_space.root_ppn().val();
        let thread = Thread::new(satp, context);
        // 复制父进程文件符描述表
        let new_fd_table: Vec<Option<Mutex<Fd>>> = self
            .fd_table
            .iter()
            .map(|fd| fd.as_ref().map(|f| Mutex::new(f.lock().clone())))
            .collect();
        Some((
            Self {
                pid,
                address_space,
                fd_table: new_fd_table,
                signal: self.signal.from_fork(),
                heap_start: self.heap_start,
                heap_end: self.heap_end,
            },
            thread,
        ))
    }

    pub fn from_elf(elf: ElfFile) -> Option<(Self, Thread)> {
        let entry = match elf.header.pt2 {
            HeaderPt2::Header64(pt2)
                if pt2.type_.as_type() == header::Type::Executable
                    && pt2.machine.as_machine() == Machine::RISC_V =>
            {
                pt2.entry_point as usize
            }
            _ => None?,
        };

        const PAGE_SIZE: usize = 1 << Sv39::PAGE_BITS;
        const PAGE_MASK: usize = PAGE_SIZE - 1;

        let mut address_space = AddressSpace::new();
        let mut max_end_mem = 0usize;  // 跟踪 ELF 加载的最高地址
        
        // 输出日志
        extern crate tg_console;
        use tg_console::log;
        log::info!("from_elf: Loading ELF, entry={:#x}", entry);
        
        for program in elf.program_iter() {
            let prog_type = program.get_type();
            log::info!("from_elf: Program header type={:?}", prog_type);
            
            if !matches!(prog_type, Ok(program::Type::Load)) {
                continue;
            }

            let off_file = program.offset() as usize;
            let len_file = program.file_size() as usize;
            let off_mem = program.virtual_addr() as usize;
            let end_mem = off_mem + program.mem_size() as usize;
            assert_eq!(off_file & PAGE_MASK, off_mem & PAGE_MASK);

            log::info!("from_elf: LOAD segment vaddr={:#x}, memsz={:#x}, end={:#x}", 
                       off_mem, program.mem_size(), end_mem);
            
            // 更新最高地址
            if end_mem > max_end_mem {
                max_end_mem = end_mem;
            }

            let mut flags: [u8; 5] = *b"U___V";
            if program.flags().is_execute() {
                flags[1] = b'X';
            }
            if program.flags().is_write() {
                flags[2] = b'W';
            }
            if program.flags().is_read() {
                flags[3] = b'R';
            }
            address_space.map(
                VAddr::new(off_mem).floor()..VAddr::new(end_mem).ceil(),
                &elf.input[off_file..][..len_file],
                off_mem & PAGE_MASK,
                parse_flags(unsafe { core::str::from_utf8_unchecked(&flags) }).unwrap(),
            );
        }
        
        // 设置堆起始地址：ELF 加载的最高地址之后的下一页，确保页对齐
        const PAGE_SIZE_FOR_HEAP: usize = 1 << Sv39::PAGE_BITS;
        let heap_start = if max_end_mem % PAGE_SIZE_FOR_HEAP == 0 {
            max_end_mem
        } else {
            ((max_end_mem / PAGE_SIZE_FOR_HEAP) + 1) * PAGE_SIZE_FOR_HEAP
        };
        log::info!("from_elf: max_end_mem={:#x}, heap_start={:#x}", max_end_mem, heap_start);
        // 映射用户栈 - 增加栈大小以支持 Linux 程序
        // 用 128 个页面 (512KB) 而不是原来的 2 个页面 (8KB)
        // 注意：我们映射到包括 0x4000000000 的那一页，以便 glibc 可以访问栈顶地址
        const STACK_PAGES: usize = 128;
        let stack = unsafe {
            alloc_zeroed(Layout::from_size_align_unchecked(
                (STACK_PAGES + 1) << Sv39::PAGE_BITS,  // 多分配一页
                1 << Sv39::PAGE_BITS,
            ))
        };
        // 调整映射范围：从 stack_bottom 映射到 stack_top+1 页
        // 栈顶地址设为 0x4000000000 所在的那一页也被映射
        let stack_top_vpn = VPN::new((1 << 26) + 1);  // 映射到 0x4000001000
        let stack_bottom_vpn = VPN::new(((1 << 26) + 1) - (STACK_PAGES + 1));
        address_space.map_extern(
            stack_bottom_vpn..stack_top_vpn,
            PPN::new(stack as usize >> Sv39::PAGE_BITS),
            build_flags("U_WRV"),
        );
        // 映射异界传送门
        map_portal(&address_space);
        let satp = (8 << 60) | address_space.root_ppn().val();
        let mut context = LocalContext::user(entry);
        
        // 初始化 Linux 风格的栈布局：argc/argv/envp/auxv
        let stack_bottom_vaddr = ((1usize << 26) - STACK_PAGES) << Sv39::PAGE_BITS;
        let stack_top_vaddr = (1usize << 26) << Sv39::PAGE_BITS;  // 0x4000000000
        let stack_phys = stack as *mut u8;
        
        // 从栈顶向下写入数据
        let mut sp = stack_top_vaddr;
        
        // 写入辅助函数：在虚拟地址 sp 处写入 usize 值
        unsafe fn push_usize(
            stack_phys: *mut u8,
            stack_bottom_vaddr: usize,
            sp: &mut usize,
            value: usize
        ) {
            *sp -= 8;
            // 计算物理偏移：虚拟地址相对于栈底的偏移
            let offset = *sp - stack_bottom_vaddr;
            *(stack_phys.add(offset) as *mut usize) = value;
        }
        
        // 1. 写入 auxiliary vector (auxv)
        // AT_NULL = 0 终止符
        unsafe {
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // AT_NULL value
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // AT_NULL type
        }
        
        // 2. 写入 envp (环境变量指针数组)
        // 只需要一个 NULL 终止符
        unsafe {
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);  // envp[0] = NULL
        }
        
        // 3. 写入 argv (参数指针数组)
        // argv[1] = NULL (终止符)
        unsafe {
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);
        }
        // argv[0] = NULL (简化处理，不传递程序名字符串)
        unsafe {
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 0);
        }
        
        // 4. 写入 argc
        unsafe {
            push_usize(stack_phys, stack_bottom_vaddr, &mut sp, 1);  // argc = 1
        }
        
        // 设置栈指针指向 argc
        *context.sp_mut() = sp;
        let thread = Thread::new(satp, context);

        Some((
            Self {
                pid: ProcId::new(),
                address_space,
                fd_table: vec![
                    // Stdin
                    Some(Mutex::new(Fd::Empty {
                        read: true,
                        write: false,
                    })),
                    // Stdout
                    Some(Mutex::new(Fd::Empty {
                        read: false,
                        write: true,
                    })),
                    // Stderr
                    Some(Mutex::new(Fd::Empty {
                        read: false,
                        write: true,
                    })),
                ],
                signal: Box::new(SignalImpl::new()),
                heap_start,
                heap_end: heap_start,  // 初始时堆为空
            },
            thread,
        ))
    }
}
