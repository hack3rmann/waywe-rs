#![allow(non_camel_case_types)]

use libc::{free, malloc, realloc};
use std::{
    ffi::{CStr, c_char, c_int, c_void},
    mem::offset_of,
    num::NonZeroU32,
    os::fd::RawFd,
    ptr,
};
use thiserror::Error;

pub type wl_display = c_void;
pub type wl_registry = c_void;
pub type wl_surface = c_void;
pub type wl_compositor = c_void;
pub type wl_proxy = c_void;
pub type wl_event_queue = c_void;

/// Represents fixed point number from libwayland backend
pub type WlFixed = wl_fixed_t;

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct wl_fixed_t(pub(crate) c_int);

impl wl_fixed_t {
    pub const fn to_double(self) -> f64 {
        // see <https://chromium.googlesource.com/external/wayland/wayland/+/refs/heads/master/src/wayland-util.h#235>
        let x = ((1023_i64 + 44_i64) << 52) + (1_i64 << 51) + self.0 as i64;
        f64::from_bits(x as u64) - (3_i64 << 43) as f64
    }

    pub const fn from_double(value: f64) -> Self {
        // see <https://chromium.googlesource.com/external/wayland/wayland/+/refs/heads/master/src/wayland-util.h#248>
        let shifted = value + (3_i64 << (51 - 8)) as f64;
        Self(shifted.to_bits() as i64 as c_int)
    }

    pub const fn to_int(self) -> i32 {
        self.0 / 256
    }

    pub const fn from_int(value: i32) -> Self {
        Self(value * 256)
    }

    pub const fn from_raw(value: c_int) -> Self {
        Self(value)
    }

    pub const fn to_raw(self) -> c_int {
        self.0
    }
}

impl From<f64> for wl_fixed_t {
    fn from(value: f64) -> Self {
        Self::from_double(value)
    }
}

impl From<wl_fixed_t> for f64 {
    fn from(value: wl_fixed_t) -> Self {
        value.to_double()
    }
}

impl From<i32> for wl_fixed_t {
    fn from(value: i32) -> Self {
        Self::from_int(value)
    }
}

impl From<wl_fixed_t> for i32 {
    fn from(value: wl_fixed_t) -> Self {
        value.to_int()
    }
}

#[repr(C)]
pub struct wl_object {
    pub interface: *const wl_interface,
    pub implementation: *const c_void,
    pub id: u32,
}

#[repr(C)]
pub struct wl_message {
    pub name: *const c_char,
    pub signature: *const c_char,
    pub types: *const *const wl_interface,
}

unsafe impl Sync for wl_message {}

#[repr(C)]
pub struct wl_array {
    pub size: usize,
    pub alloc: usize,
    pub data: *mut c_void,
}

#[derive(Debug, Error)]
#[error("wl_array copy failed")]
pub struct CopyError;

impl wl_array {
    pub const fn new() -> Self {
        Self {
            size: 0,
            alloc: 0,
            data: ptr::null_mut(),
        }
    }

    /// # Safety
    ///
    /// `this` must point to a valid unallocated [`wl_array`]
    pub const unsafe fn init(this: *mut Self) {
        unsafe { this.write(Self::new()) }
    }

    /// # Safety
    ///
    /// - `this` must point to a valid object of [`wl_array`]
    /// - `this.data` must be null or valid and allocated by calls to malloc or related functions.
    pub unsafe fn release(this: *mut Self) {
        // Safety
        // See safety for the function
        unsafe {
            free((*this).data);
        }
    }

    /// # Safety
    ///
    /// - `this` must point to a valid, allocated object
    /// - if `this.data` is not null, it must be allocated by malloc or a similar function
    pub unsafe fn add(this: *mut Self, size: usize) -> *mut c_void {
        // Safety
        // `this` is valid (see the function safety)
        let upper_bound = unsafe { (*this).size + size };
        let array_alloc = unsafe { (*this).alloc };
        let new_data;

        let mut alloc = if array_alloc > 0 { array_alloc } else { 16 };

        while alloc < upper_bound {
            alloc *= 2;
        }

        if array_alloc < alloc {
            new_data = if array_alloc > 0 {
                unsafe { realloc((*this).data, alloc) }
            } else {
                unsafe { malloc(alloc) }
            };

            if new_data.is_null() {
                return ptr::null_mut();
            }

            // Safety
            // `this` is valid (see the function safety) and new_data
            // is valid, as it was successfully allocated above
            unsafe { (*this).data = new_data };
            unsafe { (*this).alloc = alloc };
        }

        // Safety
        // `this.data` points to an allocated object (see above)
        let res = unsafe { (*this).data.byte_add((*this).size) };
        unsafe { (*this).size += size };

        res
    }

    /// # Safety
    ///
    /// - `this` and `source` must point to valid objects
    /// - this.data and source.data must point to allocated, aligned objects
    ///   and its memory areas must not overlap
    /// - source.data must be valid for read for source.size bytes
    pub unsafe fn copy(this: *mut Self, source: *mut Self) -> Result<(), CopyError> {
        let array_size = unsafe { (*this).size };
        let source_size = unsafe { (*this).size };

        if array_size < source_size {
            let add_res = unsafe { wl_array::add(this, source_size - array_size) };

            if add_res.is_null() {
                return Err(CopyError);
            }
        } else {
            unsafe { (*this).size = source_size };
        }

        // Safety
        // this.data and source.data are valid, properly aligned and don't overlap (see the function safety)
        // Code above ensures that this.data is valid for write for source.size bytes
        unsafe {
            ptr::copy_nonoverlapping((*this).data, (*source).data, source_size);
        }

        Ok(())
    }
}

impl Default for wl_array {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
pub struct wl_list {
    pub prev: *mut Self,
    pub next: *mut Self,
    pub data: [u8; 0],
}

impl wl_list {
    /// # Safety
    ///
    /// `this` must point to a valid value of [`wl_list`]
    pub const unsafe fn init(this: *mut Self) {
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this)
        };
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(this)
        };
    }

    /// # Safety
    ///
    /// both `this` and `element` must point to valid values of [`wl_list`]
    pub const unsafe fn insert(this: *mut Self, element: *mut Self) {
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this)
        };
        let this_next = unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(this_next)
        };
        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(element)
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(element)
        };
    }

    /// # Safety
    ///
    /// - `element` should point to a valid value of [`wl_list`]
    /// - `element` should have valid previous and next elements
    pub const unsafe fn remove(element: *mut Self) {
        let element_next = unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(element_next)
        };
        let element_prev = unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(element_prev)
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(ptr::null_mut())
        };
        unsafe {
            element
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(ptr::null_mut())
        };
    }

    /// # Safety
    ///
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `this` must have valid `next` and `prev` values
    pub unsafe fn length(this: *const Self) -> usize {
        let mut elem = unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        };
        let mut count = 0_usize;

        while elem.cast_const() != this {
            elem = unsafe {
                elem.wrapping_byte_add(offset_of!(wl_list, next))
                    .cast::<*mut Self>()
                    .read()
            };
            count += 1;
        }

        count
    }

    /// # Safety
    ///
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `this` must have a valid `next` value
    pub unsafe fn empty(this: *const Self) -> bool {
        this == unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
        }
    }

    /// # Safety
    ///
    /// - `this` must point to a valid value of [`wl_list`]
    /// - `other` must point to a valid value of [`wl_list`]
    /// - `this` must have a valid `next` value
    /// - `other` must have a valid `next` value
    /// - `other` must have a valid `prev` value
    pub unsafe fn insert_list(this: *mut Self, other: *mut Self) {
        if unsafe { Self::empty(other) } {
            return;
        }

        unsafe {
            other
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(this);
        }

        unsafe {
            other
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(
                    this.wrapping_byte_add(offset_of!(wl_list, next))
                        .cast::<*mut Self>()
                        .read(),
                );
        }

        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .read()
                .wrapping_byte_add(offset_of!(wl_list, prev))
                .cast::<*mut Self>()
                .write(
                    other
                        .wrapping_byte_add(offset_of!(wl_list, prev))
                        .cast::<*mut Self>()
                        .read(),
                );
        }

        unsafe {
            this.wrapping_byte_add(offset_of!(wl_list, next))
                .cast::<*mut Self>()
                .write(
                    other
                        .wrapping_byte_add(offset_of!(wl_list, next))
                        .cast::<*mut Self>()
                        .read(),
                );
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WlArgument {
    /// int
    pub i: i32,
    /// uint
    pub u: u32,
    /// fixed
    pub f: wl_fixed_t,
    /// string
    pub s: *const c_char,
    /// object
    pub o: *const wl_object,
    /// new_id
    pub n: u32,
    /// array
    pub a: *const wl_array,
    /// fd
    pub h: RawFd,
}

impl WlArgument {
    pub const fn int(value: i32) -> Self {
        Self { i: value }
    }

    pub const fn uint(value: u32) -> Self {
        Self { u: value }
    }

    pub const fn fixed(value: WlFixed) -> Self {
        Self { f: value }
    }

    pub const fn c_str(value: *const c_char) -> Self {
        Self { s: value }
    }

    pub const fn object(value: *const wl_object) -> Self {
        Self { o: value }
    }

    pub const fn new_id() -> Self {
        Self { n: 0 }
    }

    pub const fn array(value: *const wl_array) -> Self {
        Self { a: value }
    }

    pub const fn raw_fd(value: RawFd) -> Self {
        Self { h: value }
    }
}

pub type wl_argument = WlArgument;

pub type wl_dispatcher_func_t = unsafe extern "C" fn(
    *const c_void,
    *mut c_void,
    u32,
    *const wl_message,
    *mut wl_argument,
) -> c_int;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct wl_interface {
    pub name: *const c_char,
    pub version: c_int,
    pub method_count: c_int,
    pub methods: *const wl_message,
    pub event_count: c_int,
    pub events: *const wl_message,
}

unsafe impl Sync for wl_interface {}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Interface<'s> {
    pub name: &'s CStr,
    pub version: NonZeroU32,
    pub methods: &'s [InterfaceMessage<'s>],
    pub events: &'s [InterfaceMessage<'s>],
}

impl<'s> Interface<'s> {
    /// # Safety
    ///
    /// Caller ensures the interface name is a valid UTF-8 string
    pub const unsafe fn name_str_unchecked(&self) -> &'s str {
        unsafe { std::str::from_utf8_unchecked(self.name.to_bytes()) }
    }
}

#[derive(Clone, Default, Copy)]
pub struct InterfaceWlMessages<'s> {
    pub methods: &'s [wl_message],
    pub events: &'s [wl_message],
}
static_assertions::assert_impl_all!(InterfaceWlMessages<'static>: Sync);

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum OutgoingInterface<'s> {
    This,
    Other(&'s Interface<'s>),
    #[default]
    None,
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceMessage<'s> {
    pub name: &'s CStr,
    pub signature: &'s CStr,
    pub outgoing_interfaces: &'s [OutgoingInterface<'s>],
}
static_assertions::assert_impl_all!(InterfaceMessage<'static>: Sync);

impl<'s> InterfaceMessage<'s> {
    /// # Safety
    ///
    /// Caller ensures the interface name is a valid UTF-8 string
    pub const unsafe fn name_str_unchecked(&self) -> &'s str {
        unsafe { std::str::from_utf8_unchecked(self.name.to_bytes()) }
    }

    /// # Safety
    ///
    /// Caller ensures the interface name is a valid UTF-8 string
    pub const unsafe fn signature_str_unchecked(&self) -> &'s str {
        unsafe { std::str::from_utf8_unchecked(self.signature.to_bytes()) }
    }
}

pub fn count_arguments_from_bytes(bytes: impl IntoIterator<Item = u8>) -> usize {
    bytes
        .into_iter()
        .filter(|&byte| byte != b'?' && !byte.is_ascii_digit())
        .count()
}

pub fn count_arguments_from_message_signature(signature: &CStr) -> usize {
    count_arguments_from_bytes(signature.to_bytes().iter().copied())
}

/// # Safety
///
/// - `signature` should be non-null
/// - `signature` should point to a valid c-string.
pub unsafe fn count_arguments_from_message_signature_raw(signature: *const c_char) -> usize {
    count_arguments_from_bytes((0_usize..).map_while(|i| {
        let byte = unsafe { signature.wrapping_add(i).read() as u8 };
        (byte != 0).then_some(byte)
    }))
}

pub mod display_error {
    /// Operation not permitted
    pub const EPERM: i32 = 1;
    /// No such file or directory
    pub const ENOENT: i32 = 2;
    /// No such process
    pub const ESRCH: i32 = 3;
    /// Interrupted system call
    pub const EINTR: i32 = 4;
    /// I/O error
    pub const EIO: i32 = 5;
    /// No such device or address
    pub const ENXIO: i32 = 6;
    /// Argument list too long
    pub const E2BIG: i32 = 7;
    /// Exec format error
    pub const ENOEXEC: i32 = 8;
    /// Bad file number
    pub const EBADF: i32 = 9;
    /// No child processes
    pub const ECHILD: i32 = 10;
    /// Try again
    pub const EAGAIN: i32 = 11;
    /// Out of memory
    pub const ENOMEM: i32 = 12;
    /// Permission denied
    pub const EACCES: i32 = 13;
    /// Bad address
    pub const EFAULT: i32 = 14;
    /// Block device required
    pub const ENOTBLK: i32 = 15;
    /// Device or resource busy
    pub const EBUSY: i32 = 16;
    /// File exists
    pub const EEXIST: i32 = 17;
    /// Cross-device link
    pub const EXDEV: i32 = 18;
    /// No such device
    pub const ENODEV: i32 = 19;
    /// Not a directory
    pub const ENOTDIR: i32 = 20;
    /// Is a directory
    pub const EISDIR: i32 = 21;
    /// Invalid argument
    pub const EINVAL: i32 = 22;
    /// File table overflow
    pub const ENFILE: i32 = 23;
    /// Too many open files
    pub const EMFILE: i32 = 24;
    /// Not a typewriter
    pub const ENOTTY: i32 = 25;
    /// Text file busy
    pub const ETXTBSY: i32 = 26;
    /// File too large
    pub const EFBIG: i32 = 27;
    /// No space left on device
    pub const ENOSPC: i32 = 28;
    /// Illegal seek
    pub const ESPIPE: i32 = 29;
    /// Read-only file system
    pub const EROFS: i32 = 30;
    /// Too many links
    pub const EMLINK: i32 = 31;
    /// Broken pipe
    pub const EPIPE: i32 = 32;
    /// Math argument out of domain of func
    pub const EDOM: i32 = 33;
    /// Math result not representable
    pub const ERANGE: i32 = 34;
    /// Resource deadlock would occur
    pub const EDEADLK: i32 = 35;
    /// File name too long
    pub const ENAMETOOLONG: i32 = 36;
    /// No record locks available
    pub const ENOLCK: i32 = 37;
    /// Invalid system call number
    pub const ENOSYS: i32 = 38;
    /// Directory not empty
    pub const ENOTEMPTY: i32 = 39;
    /// Too many symbolic links encountered
    pub const ELOOP: i32 = 40;
    /// Operation would block
    pub const EWOULDBLOCK: i32 = EAGAIN;
    /// No message of desired type
    pub const ENOMSG: i32 = 42;
    /// Identifier removed
    pub const EIDRM: i32 = 43;
    /// Channel number out of range
    pub const ECHRNG: i32 = 44;
    /// Level 2 not synchronized
    pub const EL2NSYNC: i32 = 45;
    /// Level 3 halted
    pub const EL3HLT: i32 = 46;
    /// Level 3 reset
    pub const EL3RST: i32 = 47;
    /// Link number out of range
    pub const ELNRNG: i32 = 48;
    /// Protocol driver not attached
    pub const EUNATCH: i32 = 49;
    /// No CSI structure available
    pub const ENOCSI: i32 = 50;
    /// Level 2 halted
    pub const EL2HLT: i32 = 51;
    /// Invalid exchange
    pub const EBADE: i32 = 52;
    /// Invalid request descriptor
    pub const EBADR: i32 = 53;
    /// Exchange full
    pub const EXFULL: i32 = 54;
    /// No anode
    pub const ENOANO: i32 = 55;
    /// Invalid request code
    pub const EBADRQC: i32 = 56;
    /// Invalid slot
    pub const EBADSLT: i32 = 57;
    pub const EDEADLOCK: i32 = EDEADLK;
    /// Bad font file format
    pub const EBFONT: i32 = 59;
    /// Device not a stream
    pub const ENOSTR: i32 = 60;
    /// No data available
    pub const ENODATA: i32 = 61;
    /// Timer expired
    pub const ETIME: i32 = 62;
    /// Out of streams resources
    pub const ENOSR: i32 = 63;
    /// Machine is not on the network
    pub const ENONET: i32 = 64;
    /// Package not installed
    pub const ENOPKG: i32 = 65;
    /// Object is remote
    pub const EREMOTE: i32 = 66;
    /// Link has been severed
    pub const ENOLINK: i32 = 67;
    /// Advertise error
    pub const EADV: i32 = 68;
    /// Srmount error
    pub const ESRMNT: i32 = 69;
    /// Communication error on send
    pub const ECOMM: i32 = 70;
    /// Protocol error
    pub const EPROTO: i32 = 71;
    /// Multihop attempted
    pub const EMULTIHOP: i32 = 72;
    /// RFS specific error
    pub const EDOTDOT: i32 = 73;
    /// Not a data message
    pub const EBADMSG: i32 = 74;
    /// Value too large for defined data type
    pub const EOVERFLOW: i32 = 75;
    /// Name not unique on network
    pub const ENOTUNIQ: i32 = 76;
    /// File descriptor in bad state
    pub const EBADFD: i32 = 77;
    /// Remote address changed
    pub const EREMCHG: i32 = 78;
    /// Can not access a needed shared library
    pub const ELIBACC: i32 = 79;
    /// Accessing a corrupted shared library
    pub const ELIBBAD: i32 = 80;
    /// .lib section in a.out corrupted
    pub const ELIBSCN: i32 = 81;
    /// Attempting to link in too many shared libraries
    pub const ELIBMAX: i32 = 82;
    /// Cannot exec a shared library directly
    pub const ELIBEXEC: i32 = 83;
    /// Illegal byte sequence
    pub const EILSEQ: i32 = 84;
    /// Interrupted system call should be restarted
    pub const ERESTART: i32 = 85;
    /// Streams pipe error
    pub const ESTRPIPE: i32 = 86;
    /// Too many users
    pub const EUSERS: i32 = 87;
    /// Socket operation on non-socket
    pub const ENOTSOCK: i32 = 88;
    /// Destination address required
    pub const EDESTADDRREQ: i32 = 89;
    /// Message too long
    pub const EMSGSIZE: i32 = 90;
    /// Protocol wrong type for socket
    pub const EPROTOTYPE: i32 = 91;
    /// Protocol not available
    pub const ENOPROTOOPT: i32 = 92;
    /// Protocol not supported
    pub const EPROTONOSUPPORT: i32 = 93;
    /// Socket type not supported
    pub const ESOCKTNOSUPPORT: i32 = 94;
    /// Operation not supported on transport endpoint
    pub const EOPNOTSUPP: i32 = 95;
    /// Protocol family not supported
    pub const EPFNOSUPPORT: i32 = 96;
    /// Address family not supported by protocol
    pub const EAFNOSUPPORT: i32 = 97;
    /// Address already in use
    pub const EADDRINUSE: i32 = 98;
    /// Cannot assign requested address
    pub const EADDRNOTAVAIL: i32 = 99;
    /// Network is down
    pub const ENETDOWN: i32 = 100;
    /// Network is unreachable
    pub const ENETUNREACH: i32 = 101;
    /// Network dropped connection because of reset
    pub const ENETRESET: i32 = 102;
    /// Software caused connection abort
    pub const ECONNABORTED: i32 = 103;
    /// Connection reset by peer
    pub const ECONNRESET: i32 = 104;
    /// No buffer space available
    pub const ENOBUFS: i32 = 105;
    /// Transport endpoint is already connected
    pub const EISCONN: i32 = 106;
    /// Transport endpoint is not connected
    pub const ENOTCONN: i32 = 107;
    /// Cannot send after transport endpoint shutdown
    pub const ESHUTDOWN: i32 = 108;
    /// Too many references: cannot splice
    pub const ETOOMANYREFS: i32 = 109;
    /// Connection timed out
    pub const ETIMEDOUT: i32 = 110;
    /// Connection refused
    pub const ECONNREFUSED: i32 = 111;
    /// Host is down
    pub const EHOSTDOWN: i32 = 112;
    /// No route to host
    pub const EHOSTUNREACH: i32 = 113;
    /// Operation already in progress
    pub const EALREADY: i32 = 114;
    /// Operation now in progress
    pub const EINPROGRESS: i32 = 115;
    /// Stale file handle
    pub const ESTALE: i32 = 116;
    /// Structure needs cleaning
    pub const EUCLEAN: i32 = 117;
    /// Not a XENIX named type file
    pub const ENOTNAM: i32 = 118;
    /// No XENIX semaphores available
    pub const ENAVAIL: i32 = 119;
    /// Is a named type file
    pub const EISNAM: i32 = 120;
    /// Remote I/O error
    pub const EREMOTEIO: i32 = 121;
    /// Quota exceeded
    pub const EDQUOT: i32 = 122;
    /// No medium found
    pub const ENOMEDIUM: i32 = 123;
    /// Wrong medium type
    pub const EMEDIUMTYPE: i32 = 124;
    /// Operation Canceled
    pub const ECANCELED: i32 = 125;
    /// Required key not available
    pub const ENOKEY: i32 = 126;
    /// Key has expired
    pub const EKEYEXPIRED: i32 = 127;
    /// Key has been revoked
    pub const EKEYREVOKED: i32 = 128;
    /// Key was rejected by service
    pub const EKEYREJECTED: i32 = 129;
    /// Owner died
    pub const EOWNERDEAD: i32 = 130;
    /// State not recoverable
    pub const ENOTRECOVERABLE: i32 = 131;
    /// Operation not possible due to RF-kill
    pub const ERFKILL: i32 = 132;
    /// Memory page has hardware error
    pub const EHWPOISON: i32 = 133;
}

#[link(name = "wayland-client")]
#[allow(dead_code)]
unsafe extern "C" {
    pub static wl_display_interface: wl_interface;
    pub static wl_registry_interface: wl_interface;
    pub static wl_callback_interface: wl_interface;
    pub static wl_compositor_interface: wl_interface;
    pub static wl_shm_pool_interface: wl_interface;
    pub static wl_shm_interface: wl_interface;
    pub static wl_buffer_interface: wl_interface;
    pub static wl_data_offer_interface: wl_interface;
    pub static wl_data_source_interface: wl_interface;
    pub static wl_data_device_interface: wl_interface;
    pub static wl_data_device_manager_interface: wl_interface;
    pub static wl_shell_interface: wl_interface;
    pub static wl_shell_surface_interface: wl_interface;
    pub static wl_surface_interface: wl_interface;
    pub static wl_seat_interface: wl_interface;
    pub static wl_pointer_interface: wl_interface;
    pub static wl_keyboard_interface: wl_interface;
    pub static wl_touch_interface: wl_interface;
    pub static wl_output_interface: wl_interface;
    pub static wl_region_interface: wl_interface;
    pub static wl_subcompositor_interface: wl_interface;
    pub static wl_subsurface_interface: wl_interface;

    /// Connect to Wayland display on an already open fd.
    ///
    /// The [`wl_display`] takes ownership of the fd and will close
    /// it when the display is destroyed. The fd will also be closed in case of failure.
    pub fn wl_display_connect_to_fd(fd: RawFd) -> *mut wl_display;

    /// Close a connection to a Wayland display.
    ///
    /// Close the connection to display. The [`wl_proxy`] and `wl_event_queue`
    /// objects need to be manually destroyed by the caller before disconnecting.
    pub fn wl_display_disconnect(display: *mut wl_display);

    /// Block until all pending request are processed by the server.
    ///
    /// This function blocks until the server has processed all currently
    /// issued requests by sending a request to the display server
    /// and waiting for a reply before returning.
    ///
    /// This function uses `wl_display_dispatch_queue()` internally. It is not
    /// allowed to call this function while the thread is being prepared for
    /// reading events, and doing so will cause a dead lock.
    ///
    /// # Note
    ///
    /// This function may dispatch other events being received on the default queue.
    pub fn wl_display_roundtrip(display: *mut wl_display) -> c_int;

    /// Process incoming events
    /// Dispatch events on the default event queue.
    ///
    /// # Parameters
    ///
    /// - `display` - The display context object
    ///
    /// # Returns
    ///
    /// The number of dispatched events on success or `-1` on failure
    ///
    /// If the default event queue is empty, this function blocks until there are
    /// events to be read from the display fd. Events are read and queued on the
    /// appropriate event queues. Finally, events on the default event queue are
    /// dispatched. On failure `-1` is returned and errno set appropriately.
    ///
    /// In a multi threaded environment, do not manually wait using poll()
    /// (or equivalent) before calling this function, as doing so might cause a
    /// dead lock. If external reliance on poll() (or equivalent) is required,
    /// see wl_display_prepare_read_queue() of how to do so.
    ///
    /// This function is thread safe as long as it dispatches the right queue
    /// on the right thread. It is also compatible with the multi thread event
    /// reading preparation API (see wl_display_prepare_read_queue()), and uses
    /// the equivalent functionality internally. It is not allowed to call this
    /// function while the thread is being prepared for reading events, and doing so will cause a dead lock.
    ///
    /// # Note
    ///
    /// It is not possible to check if there are events on the queue or not. For
    /// dispatching default queue events without blocking, see wl_display_dispatch_pending().
    ///
    /// # See also
    ///
    /// - `wl_display_dispatch_pending()`
    /// - `wl_display_dispatch_queue()`
    /// - `wl_display_read_events()`
    pub fn wl_display_dispatch(display: *mut wl_display) -> c_int;

    /// Create a new event queue for this display
    ///
    /// # Parameters
    ///
    /// display The display context object
    ///
    /// # Returns
    ///
    /// A new event queue associated with this display or NULL on failure.
    pub fn wl_display_create_queue(display: *mut wl_display) -> *mut wl_event_queue;

    /// Create a new event queue for this display and give it a name
    ///
    /// # Parameters
    ///
    /// display The display context object
    /// name A human readable queue name
    ///
    /// # Returns
    ///
    /// A new event queue associated with this display or NULL on failure.
    pub fn wl_display_create_queue_with_name(
        display: *mut wl_display,
        name: *const c_char,
    ) -> *mut wl_event_queue;

    /// Retrieve the last error that occurred on a display
    ///
    /// # Parameters
    ///
    /// - `display` - The display context object
    ///
    /// # Returns
    ///
    /// The last error that occurred on display or 0 if no error occurred
    ///
    /// Return the last error that occurred on the display. This may be an error sent
    /// by the server or caused by the local client.
    ///
    /// # Note
    ///
    /// Errors are fatal. If this function returns non-zero the display
    /// can no longer be used.
    pub fn wl_display_get_error(display: *mut wl_display) -> c_int;

    /// Destroy an event queue
    ///
    /// # Parameters
    ///
    /// queue The event queue to be destroyed
    ///
    /// Destroy the given event queue. Any pending event on that queue is discarded.
    ///
    /// The wl_display object used to create the queue should not be destroyed until
    /// all event queues created with it are destroyed with this function.
    pub fn wl_event_queue_destroy(queue: *mut wl_event_queue);

    /// Get the protocol object version of a proxy object.
    ///
    /// Gets the protocol object version of a proxy object, or `0`
    /// if the proxy was created with unversioned API.
    ///
    /// A returned value of `0` means that no version information is available,
    /// so the caller must make safe assumptions about the object's real version.
    ///
    /// [`wl_display`]'s version will always return `0`.
    pub fn wl_proxy_get_version(proxy: *mut wl_proxy) -> u32;

    /// Prepare a request to be sent to the compositor.
    ///
    /// # Params
    ///
    /// - `proxy` - The proxy object
    /// - `opcode` - Opcode of the request to be sent
    /// - `interface` - The interface to use for the new proxy
    /// - `version` - The protocol object version of the new proxy
    /// - `flags` - Flags that modify marshalling behaviour
    /// - `...` - Extra arguments for the given request
    ///
    /// # Return value
    ///
    /// A new [`wl_proxy`] for the `new_id` argument or [`ptr::null_mut`] on error
    ///
    /// Translates the request given by `opcode` and the extra arguments into the
    /// wire format and write it to the connection buffer.
    ///
    /// For new-id arguments, this function will allocate a new [`wl_proxy`] and send
    /// the ID to the server. The new [`wl_proxy`] will be returned on success or NULL
    /// on error with errno set accordingly. The newly created proxy will have
    /// the version specified.
    ///
    /// The flag `WL_MARSHAL_FLAG_DESTROY` may be passed to ensure the proxy is
    /// destroyed atomically with the marshalling in order to prevent races that
    /// can occur if the display lock is dropped between the marshal and destroy
    /// operations.
    ///
    /// # Note
    ///
    /// This should not normally be used by non-generated code.
    pub fn wl_proxy_marshal_flags(
        proxy: *mut wl_proxy,
        opcode: u32,
        interface: *const wl_interface,
        version: u32,
        flags: u32,
        ...
    ) -> *mut wl_proxy;

    /// Prepare a request to be sent to the compositor
    ///
    /// # Parameters
    ///
    /// - `proxy` - The proxy object
    /// - `opcode` - Opcode of the request to be sent
    /// - `args` - Extra arguments for the given request
    /// - `interface` - The interface to use for the new proxy
    ///
    /// This function translates a request given an opcode, an interface and
    /// a wl_argument array to the wire format and writes it to the connection buffer.
    ///
    /// For new-id arguments, this function will allocate a new [`wl_proxy`] and send
    /// the ID to the server. The new [`wl_proxy`] will be returned on success or NULL
    /// on error with errno set accordingly. The newly created proxy will inherit
    /// their version from their parent.
    ///
    /// # Note
    ///
    /// This is intended to be used by language bindings and not in non-generated code.
    ///
    /// # See also
    ///
    /// `wl_proxy_marshal()`
    pub fn wl_proxy_marshal_array_constructor(
        proxy: *mut wl_proxy,
        opcode: u32,
        args: *mut wl_argument,
        interface: *const wl_interface,
    ) -> *mut wl_proxy;

    /// Destroy a proxy object.
    ///
    /// # Safety
    ///
    /// `proxy` must not be a proxy wrapper.
    ///
    /// # Note
    ///
    /// This function will abort in response to egregious errors, and will do so
    /// with the display lock held. This means SIGABRT handlers must not perform
    /// any actions that would attempt to take that lock, or a deadlock would occur.
    pub fn wl_proxy_destroy(proxy: *mut wl_proxy);

    /// Set a proxy's listener.
    ///
    /// `proxy` must not be a proxy wrapper.
    ///
    /// # Note
    ///
    /// This function will abort in response to egregious errors, and will do
    /// so with the display lock held. This means SIGABRT handlers must not perform
    /// any actions that would attempt to take that lock, or a deadlock would occur.
    pub fn wl_proxy_add_listener(
        proxy: *mut wl_proxy,
        implementation: *mut extern "C" fn(),
        data: *mut c_void,
    ) -> c_int;

    /// Set a proxy's listener (with dispatcher)
    ///
    /// # Parameters
    ///
    /// - `proxy` - The proxy object
    /// - `dispatcher` - The dispatcher to be used for this proxy
    /// - `implementation` - The dispatcher-specific listener implementation
    /// - `data` - User data to be associated with the proxy
    ///
    /// # Returns
    ///
    /// `0` on success or `-1` on failure
    ///
    /// Set proxy's listener to use `dispatcher` as its dispatcher and
    /// `data` as its dispatcher-specific implementation and its user
    /// data to data. If a listener has already been set, this function fails
    /// and nothing is changed.
    ///
    /// The exact details of dispatcher_data depend on the dispatcher used. This
    /// function is intended to be used by language bindings, not user code.
    ///
    /// # Safety
    ///
    /// `proxy` must not be a proxy wrapper.
    pub fn wl_proxy_add_dispatcher(
        proxy: *mut wl_proxy,
        dispatcher: wl_dispatcher_func_t,
        implementation: *const c_void,
        data: *mut c_void,
    ) -> c_int;

    /// Get the id of a proxy object.
    pub fn wl_proxy_get_id(proxy: *mut wl_proxy) -> u32;

    /// Get the interface name (class) of a proxy object
    pub fn wl_proxy_get_class(proxy: *mut wl_proxy) -> *const c_char;

    /// Get user data associated woth a proxy
    pub fn wl_proxy_get_user_data(proxy: *mut wl_proxy) -> *mut c_void;

    /// Set the user data associated with a proxy
    pub fn wl_proxy_set_user_data(proxy: *mut wl_proxy, data: *mut c_void);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_signature_and_types() {
        let interface = unsafe { &wl_data_device_interface };

        let enter_event = unsafe { interface.events.add(1).read() };
        let signature = unsafe { CStr::from_ptr(enter_event.signature) };

        assert_eq!(signature, c"uoff?o");
        assert!(!enter_event.types.is_null());

        let n_args = count_arguments_from_message_signature(signature);

        let types = unsafe { std::slice::from_raw_parts(enter_event.types, n_args) };

        assert_eq!(
            types,
            &[
                ptr::null(),
                &raw const wl_surface_interface,
                ptr::null(),
                ptr::null(),
                &raw const wl_data_offer_interface,
            ]
        );
    }
}
