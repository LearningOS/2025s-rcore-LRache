//! Process management syscalls
use crate::config::PAGE_SIZE;
use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, get_count_syscall, progress_mmap, progress_unmap, suspend_current_and_run_next};
use crate::mm::{copy_from_kernel, copy_from_user, MapPermission};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let now = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    match copy_from_kernel(
        current_user_token(), 
        ts as usize, 
        &now as *const TimeVal as *const u8, 
        core::mem::size_of_val(&now)
    ) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let mut v: u8 = 0;
            match copy_from_user(
                current_user_token(), 
                &mut v as *mut u8,
                id,
                1
            ) {
                Ok(_) => return v as isize,
                Err(_) => return -1,
            }
        },

        1 => {
            let v = data as u8;
            match copy_from_kernel(
                current_user_token(), 
                id, 
                &v as *const u8, 
                1
            ) {
                Ok(_) => return 0,
                Err(_) => return -1,
            }
        },

        2 => {
            return get_count_syscall(id) as isize;
        }

        _ => return -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap start: {:#x}, len: {:#x}, port: {:#x}", start, len, port);
    if port & 0x7 == 0 {
        return -1;
    }
    if port & !0x7 != 0 {
        return -1;
    }
    if start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    
    let mut p = MapPermission::empty();
    p |= MapPermission::U;
    if port & (1 << 0) != 0 {
        p |= MapPermission::R;
    }
    if port & (1 << 1) != 0 {
        p |= MapPermission::W;
    }
    if port & (1 << 2) != 0 {
        p |= MapPermission::X;
    }

    match progress_mmap(start, len, p) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    match progress_unmap(start, len) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
