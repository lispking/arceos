use core::{ptr::NonNull, alloc::Layout};

use crate::{ByteAllocator, AllocError, BaseAllocator};

pub struct EarlyByteAllocator {
    start: usize,
    pos: usize,
    total_bytes: usize,
    used_bytes: usize,
}

impl EarlyByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            pos: 0,
            total_bytes: 0,
            used_bytes: 0,
        }
    }

    #[inline]
    fn increase_pos(&mut self, size: usize) {
        self.pos += size;
    }

    #[inline]
    fn reset_pos(&mut self) {
        if self.used_bytes == 0 {
            self.pos = self.start;
        }
    }
}

impl BaseAllocator for EarlyByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.pos = start;
        self.total_bytes = size;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> crate::AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl ByteAllocator for EarlyByteAllocator {
    fn alloc(&mut self, layout: Layout) -> crate::AllocResult<NonNull<u8>> {
        match NonNull::new(self.pos as *mut u8) {
            Some(pos) => {
                let size = layout.size();
                self.used_bytes += size;
                self.increase_pos(size);
                Ok(pos)
            }
            None => {
                Err(AllocError::NoMemory)
            }
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        self.used_bytes -= size;
        self.reset_pos();
    }

    fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.total_bytes - self.used_bytes
    }
}