use crate::intrinsics;
use crate::os::raw::c_char;

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
pub mod fd;
pub mod fs;
pub mod futex;
#[path = "../unsupported/io.rs"]
pub mod io;
pub mod memchr;
#[path = "../unsupported/net.rs"]
pub mod net;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
pub mod stdio;
pub mod thread;
pub mod thread_local_dtor;
#[path = "../unsupported/thread_local_key.rs"]
pub mod thread_local_key;
pub mod time;

// We can use unix locks by providing futex.
#[path = "../unix/locks"]
pub mod locks {
    mod futex_condvar;
    mod futex_mutex;
    mod futex_rwlock;
    pub(crate) use futex_condvar::Condvar;
    pub(crate) use futex_mutex::Mutex;
    pub(crate) use futex_rwlock::RwLock;
}

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::const_io_error!(
        crate::io::ErrorKind::Unsupported,
        &"operation not supported on Twizzler yet",
    )
}

pub unsafe fn strlen(start: *const c_char) -> usize {
    let mut str = start;

    while *str != 0 {
        str = str.offset(1);
    }

    (str as usize) - (start as usize)
}

#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    unsafe { compiler_builtins::math::floor(x) }
}

#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    unsafe { compiler_builtins::math::ceil(x) }
}

#[no_mangle]
pub extern "C" fn log2(x: f64) -> f64 {
    unsafe { compiler_builtins::math::log2(x) }
}

#[inline]
pub fn abort_internal() -> ! {
    let runtime = twizzler_runtime_api::get_runtime();
    runtime.abort()
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    abort_internal();
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    args::init(argc, argv);
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

#[inline]
pub(crate) fn is_interrupted(errno: i32) -> bool {
    false
}

pub fn decode_error_kind(_errno: i32) -> crate::io::ErrorKind {
    unimplemented!()
}

// FIXME: just a workaround to test the system
pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}

#[no_mangle]
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub unsafe extern "C" fn std_entry_from_runtime(
    aux: twizzler_runtime_api::BasicAux,
) -> twizzler_runtime_api::BasicReturn {
    extern "C" {
        fn main(argc: isize, argv: *const *const c_char) -> i32;
    }

    crate::sys::os::init_environment(aux.env);
    let runtime = twizzler_runtime_api::get_runtime();
    runtime.pre_main_hook();
    let code = main(aux.argc as isize, aux.args);

    // If pre_main_hook returns a code, then don't call main and exit with that code instead.
    //let code = if let Some(pre_code) = runtime.pre_main_hook() {
    //    pre_code
    //} else {
    //    main(aux.argc as isize, aux.args)
    //};
    runtime.post_main_hook();

    thread_local_dtor::run_dtors();
    twizzler_runtime_api::BasicReturn { code }
}

// When in minruntime, we expect this symbol to be provided by twizzler-abi, and we further require
// applications for minruntime to link explicitly to a provider of this symbol. For reference runtime,
// we need to provide this for binaries to link properly. The loader will ensure the correct definition
// is used because we mark this as weak. The provider (twz_rt) exports this as a strong (normal) symbol.
#[cfg(not(target_env = "minruntime"))]
pub mod __runtime_fallback_imp {
    #[linkage = "weak"]
    #[no_mangle]
    pub unsafe extern "C" fn __twz_get_runtime() {
        core::intrinsics::abort()
    }
}
