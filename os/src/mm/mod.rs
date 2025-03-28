//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
use crate::config::PAGE_SIZE;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}

/// Copy from user memory space to kernel
pub fn copy_from_user(token: usize, dest: *mut u8, src: usize, len: usize) -> Result<(), ()> {
    let page_table = PageTable::from_token(token);
    let mut start_va = src;
    
    let mut left = len;
    let mut copied: usize = 0;
    
    while left != 0 {
        let vaddr = VirtAddr::from(start_va);
        let vpn = vaddr.floor();
        if let Some(pte) = page_table.translate(vpn) {
            if !(pte.is_valid() && pte.readable() && pte.user_accessiable()) {
                return Err(());
            }
            
            let ppn = pte.ppn();
            let count = left.min(PAGE_SIZE - vaddr.page_offset());
            
            unsafe {
                core::ptr::copy_nonoverlapping(
                    ppn.get_bytes_array().as_ptr().add(vaddr.page_offset()), 
                    dest.add(copied), 
                    count
                );
            }
            
            copied += count;
            start_va += count;
            left -= count;
        
        } else {
            return Err(())
        }
    }
    Ok(())
}

/// Copy from kernel memory space to user
pub fn copy_from_kernel(token: usize, dest: usize, src: *const u8, len:usize) -> Result<(), ()> {
    let page_table = PageTable::from_token(token);
    let mut start_va = dest;

    let mut left = len;
    let mut copied: usize = 0;
    
    while left != 0 {
        let vaddr = VirtAddr::from(start_va);
        let vpn = vaddr.floor();
        if let Some(pte) = page_table.translate(vpn) {
            if !pte.writable() {
                return Err(());
            }

            let ppn = pte.ppn();
            let count = left.min(PAGE_SIZE - vaddr.page_offset());

            unsafe {
                core::ptr::copy_nonoverlapping(
                    src.add(copied), 
                    ppn.get_bytes_array().as_mut_ptr().add(vaddr.page_offset()), 
                    count
                );
            }

            copied += count;
            left -= count;
            start_va += count;
        } else {
            return Err(());
        }
    }
    
    return Ok(());
}
