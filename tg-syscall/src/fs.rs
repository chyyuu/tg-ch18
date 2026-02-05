/// 文件类型/权限标志（与 Linux 约定保持一致）。
pub struct StatMode;

impl StatMode {
    /// file type mask
    pub const S_IFMT: u32 = 0o170000;
    /// regular file
    pub const S_IFREG: u32 = 0o100000;
    /// directory
    pub const S_IFDIR: u32 = 0o040000;
    /// default file permissions (rw-r--r--)
    pub const DEFAULT_FILE_PERM: u32 = 0o644;
    /// default directory permissions (rwxr-xr-x)
    pub const DEFAULT_DIR_PERM: u32 = 0o755;
}

/// 文件状态信息（最小 Linux 兼容子集）。
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_size: i64,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_size: 0,
        }
    }
}
