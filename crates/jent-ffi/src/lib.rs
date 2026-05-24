use core::ffi::c_int;
use jent_core::{JentCollector, Sha3_512Conditioner, Shake256Conditioner};
use jent_platform::RdtscTimer;

#[repr(C)]
pub enum jent_conditioner_kind {
    JentConditionerSha3_512 = 1,
    JentConditionerShake256 = 2,
}

pub struct CollectorHandle {
    collector: JentCollector<RdtscTimer>,
}

#[no_mangle]
pub extern "C" fn jent_entropy_init() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn jent_entropy_collector_alloc() -> *mut CollectorHandle {
    let handle = CollectorHandle {
        collector: JentCollector::new(RdtscTimer),
    };
    Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub extern "C" fn jent_entropy_collector_free(ptr: *mut CollectorHandle) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub extern "C" fn jent_read_entropy_sha3_512(
    ptr: *mut CollectorHandle,
    out: *mut u8,
    out_len: usize,
) -> isize {
    if ptr.is_null() || out.is_null() {
        return -1;
    }

    let handle = unsafe { &mut *ptr };
    let out = unsafe { core::slice::from_raw_parts_mut(out, out_len) };
    let mut conditioner = Sha3_512Conditioner::new();

    match handle.collector.fill_conditioned(&mut conditioner, out) {
        Ok(()) => out_len as isize,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn jent_read_entropy_shake256(
    ptr: *mut CollectorHandle,
    out: *mut u8,
    out_len: usize,
) -> isize {
    if ptr.is_null() || out.is_null() {
        return -1;
    }

    let handle = unsafe { &mut *ptr };
    let out = unsafe { core::slice::from_raw_parts_mut(out, out_len) };
    let mut conditioner = Shake256Conditioner::new(64);

    match handle.collector.fill_conditioned(&mut conditioner, out) {
        Ok(()) => out_len as isize,
        Err(_) => -2,
    }
}
