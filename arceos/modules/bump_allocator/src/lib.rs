#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    /// The left bound of the whole memory region.
    start: usize,
    /// The right bound of the whole memory region.
    end: usize,
    /// The next free byte; bytes grow from `start` towards `end`.
    b_pos: usize,
    /// The lowest used page base; pages grow from `end` towards `start`.
    p_pos: usize,
    /// Number of live byte allocations.
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

/// Rounds `addr` up to the next multiple of `align`; `align` must be a power of two.
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Rounds `addr` down to the previous multiple of `align`; `align` must be a power of two.
const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // Pages are handed out from the tail of the range, so only a region
        // contiguous with `end` can be absorbed. Anything else cannot be
        // represented by a single range and is deliberately ignored.
        if start == self.end {
            self.end += size;
            self.p_pos = self.end;
        }
        Ok(())
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let layout = layout.pad_to_align();
        let start = align_up(self.b_pos, layout.align());
        let end = start + layout.size();
        if end > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        self.b_pos = end;
        self.count += 1;
        NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        self.count = self.count.saturating_sub(1);
        // A bump allocator cannot free individual chunks: the byte area is
        // reclaimed only once every allocation has been released.
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if num_pages == 0 || !align_pow2.is_power_of_two() || align_pow2 % SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let size = num_pages * SIZE;
        if size > self.p_pos - self.b_pos {
            return Err(AllocError::NoMemory);
        }
        // Grow downwards, keeping the base aligned to `align_pow2`.
        let start = align_down(self.p_pos - size, align_pow2);
        if start < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        self.p_pos = start;
        Ok(start)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Allocated pages are never returned to the page area.
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / SIZE
    }
}
