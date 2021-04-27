/* automatically generated by rust-bindgen 0.58.1 */

pub const __BITS_PER_LONG: u32 = 64;
pub const __FD_SETSIZE: u32 = 1024;
pub const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;
pub const LANDLOCK_ACCESS_FS_EXECUTE: u32 = 1;
pub const LANDLOCK_ACCESS_FS_WRITE_FILE: u32 = 2;
pub const LANDLOCK_ACCESS_FS_READ_FILE: u32 = 4;
pub const LANDLOCK_ACCESS_FS_READ_DIR: u32 = 8;
pub const LANDLOCK_ACCESS_FS_REMOVE_DIR: u32 = 16;
pub const LANDLOCK_ACCESS_FS_REMOVE_FILE: u32 = 32;
pub const LANDLOCK_ACCESS_FS_MAKE_CHAR: u32 = 64;
pub const LANDLOCK_ACCESS_FS_MAKE_DIR: u32 = 128;
pub const LANDLOCK_ACCESS_FS_MAKE_REG: u32 = 256;
pub const LANDLOCK_ACCESS_FS_MAKE_SOCK: u32 = 512;
pub const LANDLOCK_ACCESS_FS_MAKE_FIFO: u32 = 1024;
pub const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u32 = 2048;
pub const LANDLOCK_ACCESS_FS_MAKE_SYM: u32 = 4096;
pub type __s8 = ::std::os::raw::c_schar;
pub type __u8 = ::std::os::raw::c_uchar;
pub type __s16 = ::std::os::raw::c_short;
pub type __u16 = ::std::os::raw::c_ushort;
pub type __s32 = ::std::os::raw::c_int;
pub type __u32 = ::std::os::raw::c_uint;
pub type __s64 = ::std::os::raw::c_longlong;
pub type __u64 = ::std::os::raw::c_ulonglong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __kernel_fd_set {
    pub fds_bits: [::std::os::raw::c_ulong; 16usize],
}
#[test]
fn bindgen_test_layout___kernel_fd_set() {
    assert_eq!(
        ::std::mem::size_of::<__kernel_fd_set>(),
        128usize,
        concat!("Size of: ", stringify!(__kernel_fd_set))
    );
    assert_eq!(
        ::std::mem::align_of::<__kernel_fd_set>(),
        8usize,
        concat!("Alignment of ", stringify!(__kernel_fd_set))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<__kernel_fd_set>())).fds_bits as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(__kernel_fd_set),
            "::",
            stringify!(fds_bits)
        )
    );
}
pub type __kernel_sighandler_t =
    ::std::option::Option<unsafe extern "C" fn(arg1: ::std::os::raw::c_int)>;
pub type __kernel_key_t = ::std::os::raw::c_int;
pub type __kernel_mqd_t = ::std::os::raw::c_int;
pub type __kernel_old_uid_t = ::std::os::raw::c_ushort;
pub type __kernel_old_gid_t = ::std::os::raw::c_ushort;
pub type __kernel_old_dev_t = ::std::os::raw::c_ulong;
pub type __kernel_long_t = ::std::os::raw::c_long;
pub type __kernel_ulong_t = ::std::os::raw::c_ulong;
pub type __kernel_ino_t = __kernel_ulong_t;
pub type __kernel_mode_t = ::std::os::raw::c_uint;
pub type __kernel_pid_t = ::std::os::raw::c_int;
pub type __kernel_ipc_pid_t = ::std::os::raw::c_int;
pub type __kernel_uid_t = ::std::os::raw::c_uint;
pub type __kernel_gid_t = ::std::os::raw::c_uint;
pub type __kernel_suseconds_t = __kernel_long_t;
pub type __kernel_daddr_t = ::std::os::raw::c_int;
pub type __kernel_uid32_t = ::std::os::raw::c_uint;
pub type __kernel_gid32_t = ::std::os::raw::c_uint;
pub type __kernel_size_t = __kernel_ulong_t;
pub type __kernel_ssize_t = __kernel_long_t;
pub type __kernel_ptrdiff_t = __kernel_long_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __kernel_fsid_t {
    pub val: [::std::os::raw::c_int; 2usize],
}
#[test]
fn bindgen_test_layout___kernel_fsid_t() {
    assert_eq!(
        ::std::mem::size_of::<__kernel_fsid_t>(),
        8usize,
        concat!("Size of: ", stringify!(__kernel_fsid_t))
    );
    assert_eq!(
        ::std::mem::align_of::<__kernel_fsid_t>(),
        4usize,
        concat!("Alignment of ", stringify!(__kernel_fsid_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<__kernel_fsid_t>())).val as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(__kernel_fsid_t),
            "::",
            stringify!(val)
        )
    );
}
pub type __kernel_off_t = __kernel_long_t;
pub type __kernel_loff_t = ::std::os::raw::c_longlong;
pub type __kernel_old_time_t = __kernel_long_t;
pub type __kernel_time_t = __kernel_long_t;
pub type __kernel_time64_t = ::std::os::raw::c_longlong;
pub type __kernel_clock_t = __kernel_long_t;
pub type __kernel_timer_t = ::std::os::raw::c_int;
pub type __kernel_clockid_t = ::std::os::raw::c_int;
pub type __kernel_caddr_t = *mut ::std::os::raw::c_char;
pub type __kernel_uid16_t = ::std::os::raw::c_ushort;
pub type __kernel_gid16_t = ::std::os::raw::c_ushort;
pub type __le16 = __u16;
pub type __be16 = __u16;
pub type __le32 = __u32;
pub type __be32 = __u32;
pub type __le64 = __u64;
pub type __be64 = __u64;
pub type __sum16 = __u16;
pub type __wsum = __u32;
pub type __poll_t = ::std::os::raw::c_uint;
#[doc = " struct landlock_ruleset_attr - Ruleset definition"]
#[doc = ""]
#[doc = " Argument of sys_landlock_create_ruleset().  This structure can grow in"]
#[doc = " future versions."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct landlock_ruleset_attr {
    #[doc = " @handled_access_fs: Bitmask of actions (cf. `Filesystem flags`_)"]
    #[doc = " that is handled by this ruleset and should then be forbidden if no"]
    #[doc = " rule explicitly allow them.  This is needed for backward"]
    #[doc = " compatibility reasons."]
    pub handled_access_fs: __u64,
}
#[test]
fn bindgen_test_layout_landlock_ruleset_attr() {
    assert_eq!(
        ::std::mem::size_of::<landlock_ruleset_attr>(),
        8usize,
        concat!("Size of: ", stringify!(landlock_ruleset_attr))
    );
    assert_eq!(
        ::std::mem::align_of::<landlock_ruleset_attr>(),
        8usize,
        concat!("Alignment of ", stringify!(landlock_ruleset_attr))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<landlock_ruleset_attr>())).handled_access_fs as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(landlock_ruleset_attr),
            "::",
            stringify!(handled_access_fs)
        )
    );
}
#[doc = " @LANDLOCK_RULE_PATH_BENEATH: Type of a &struct"]
#[doc = " landlock_path_beneath_attr ."]
pub const landlock_rule_type_LANDLOCK_RULE_PATH_BENEATH: landlock_rule_type = 1;
#[doc = " enum landlock_rule_type - Landlock rule type"]
#[doc = ""]
#[doc = " Argument of sys_landlock_add_rule()."]
pub type landlock_rule_type = ::std::os::raw::c_uint;
#[doc = " struct landlock_path_beneath_attr - Path hierarchy definition"]
#[doc = ""]
#[doc = " Argument of sys_landlock_add_rule()."]
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct landlock_path_beneath_attr {
    #[doc = " @allowed_access: Bitmask of allowed actions for this file hierarchy"]
    #[doc = " (cf. `Filesystem flags`_)."]
    pub allowed_access: __u64,
    #[doc = " @parent_fd: File descriptor, open with ``O_PATH``, which identifies"]
    #[doc = " the parent directory of a file hierarchy, or just a file."]
    pub parent_fd: __s32,
}
#[test]
fn bindgen_test_layout_landlock_path_beneath_attr() {
    assert_eq!(
        ::std::mem::size_of::<landlock_path_beneath_attr>(),
        12usize,
        concat!("Size of: ", stringify!(landlock_path_beneath_attr))
    );
    assert_eq!(
        ::std::mem::align_of::<landlock_path_beneath_attr>(),
        1usize,
        concat!("Alignment of ", stringify!(landlock_path_beneath_attr))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<landlock_path_beneath_attr>())).allowed_access as *const _
                as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(landlock_path_beneath_attr),
            "::",
            stringify!(allowed_access)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<landlock_path_beneath_attr>())).parent_fd as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(landlock_path_beneath_attr),
            "::",
            stringify!(parent_fd)
        )
    );
}
