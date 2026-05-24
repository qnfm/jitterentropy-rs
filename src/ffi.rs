#![cfg(feature = "ffi")]

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{
    ffi::{c_char, c_int, c_uint},
    ptr, slice,
};

use crate::{collector::EntropyCollector, flags::Flags, JENT_VERSION};

#[repr(C)]
pub struct rand_data {
    inner: EntropyCollector,
}

#[no_mangle]
pub extern "C" fn jent_version() -> c_uint {
    JENT_VERSION as c_uint
}

#[no_mangle]
pub extern "C" fn jent_entropy_init() -> c_int {
    jent_entropy_init_ex(1, 0)
}

#[no_mangle]
pub extern "C" fn jent_entropy_init_ex(osr: c_uint, flags: c_uint) -> c_int {
    match EntropyCollector::new(osr as u32, Flags(flags as u32)) {
        Ok(_) => 0,
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub extern "C" fn jent_entropy_collector_alloc(osr: c_uint, flags: c_uint) -> *mut rand_data {
    match EntropyCollector::new(osr as u32, Flags(flags as u32)) {
        Ok(inner) => Box::into_raw(Box::new(rand_data { inner })),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn jent_entropy_collector_free(ec: *mut rand_data) {
    if !ec.is_null() {
        unsafe {
            drop(Box::from_raw(ec));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn jent_read_entropy(
    ec: *mut rand_data,
    data: *mut c_char,
    len: usize,
) -> isize {
    if ec.is_null() || data.is_null() {
        return -1;
    }
    let ec = unsafe { &mut *ec };
    let out = unsafe { slice::from_raw_parts_mut(data.cast::<u8>(), len) };
    match ec.inner.fill_bytes(out) {
        Ok(()) => len as isize,
        Err(e) => e.c_code(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn jent_read_entropy_safe(
    ecp: *mut *mut rand_data,
    data: *mut c_char,
    len: usize,
) -> isize {
    if ecp.is_null() {
        return -1;
    }
    let ec = unsafe { *ecp };
    let rc = unsafe { jent_read_entropy(ec, data, len) };
    if rc >= 0 {
        return rc;
    }

    if !ec.is_null() {
        unsafe {
            jent_entropy_collector_free(ec);
        }
        unsafe {
            *ecp = ptr::null_mut();
        }
    }
    let new_ec = jent_entropy_collector_alloc(1, 0);
    if new_ec.is_null() {
        return rc;
    }
    unsafe {
        *ecp = new_ec;
    }
    unsafe { jent_read_entropy(new_ec, data, len) }
}

#[no_mangle]
pub unsafe extern "C" fn jent_status(
    ec: *const rand_data,
    buf: *mut c_char,
    buflen: usize,
) -> c_int {
    if ec.is_null() || buf.is_null() || buflen == 0 {
        return -1;
    }
    #[cfg(feature = "alloc")]
    {
        let status = unsafe { &*ec }.inner.status().to_json();
        let bytes = status.as_bytes();
        let n = core::cmp::min(bytes.len(), buflen.saturating_sub(1));
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buf.cast::<u8>(), n);
            *buf.add(n) = 0;
        }
        return n as c_int;
    }
    #[cfg(not(feature = "alloc"))]
    {
        -1
    }
}

#[no_mangle]
pub extern "C" fn jent_secure_memory_supported() -> c_int {
    0
}
