//! Early allocation in page-granularity.
//!
//! TODO: adaptive size

use crate::{AllocError, AllocResult, BaseAllocator, PageAllocator};

/// A page-granularity memory allocator based on the [bitmap_allocator].
///
/// It internally uses a bitmap, each bit indicates whether a page has been
/// allocated.
///
/// The `PAGE_SIZE` must be a power of two.
///
/// [bitmap_allocator]: https://github.com/rcore-os/bitmap-allocator
pub struct EarlyPageAllocator<const PAGE_SIZE: usize> {
    end: usize,
    pos: usize,
    total_pages: usize,
    used_pages: usize,
}

impl<const PAGE_SIZE: usize> EarlyPageAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyPageAllocator`.
    pub const fn new() -> Self {
        Self {
            end: 0,
            pos: 0,
            total_pages: 0,
            used_pages: 0,
        }
    }

    #[inline]
    pub fn decrease_pos(&mut self, num_pages: usize) {
        self.pos -= PAGE_SIZE * num_pages;
    }

    #[inline]
    pub fn reset_pos(&mut self) {
        if self.used_pages == 0 {
            self.pos = self.end;
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyPageAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());

        self.end = super::align_down(start + size, PAGE_SIZE);
        let start = super::align_up(start, PAGE_SIZE);
        self.pos = self.end;
        self.total_pages = (self.end - start) / PAGE_SIZE;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyPageAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => Some(self.pos - PAGE_SIZE),
            core::cmp::Ordering::Greater => Some(self.pos - PAGE_SIZE * num_pages),
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| {
            self.used_pages += num_pages;
            self.decrease_pos(num_pages);
        })
    }

    fn dealloc_pages(&mut self, _pos: usize, num_pages: usize) {
        // TODO: not decrease `used_pages` if deallocation failed
        self.used_pages -= num_pages;
        self.reset_pos();
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.used_pages
    }
}
