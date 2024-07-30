use std::{
    alloc::{Allocator, Layout},
    cell::Cell,
    ops::Deref,
};

pub struct ArenaInner {
    inner: *mut u8,
    capacity: usize,
    cur: Cell<usize>,
}

impl !Sync for ArenaInner {}

impl ArenaInner {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            inner: unsafe { std::alloc::alloc(Layout::from_size_align(capacity, 1).unwrap()) },
            capacity,
            cur: Cell::new(0),
        }
    }

    /// All allocations are invalidated when using this function
    pub fn reuse(&mut self) -> &mut Self {
        self.cur.replace(0);
        self
    }
}

impl Drop for ArenaInner {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.inner,
                Layout::from_size_align(self.capacity, 1).unwrap(),
            )
        }
    }
}

unsafe impl Allocator for ArenaInner {
    fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        if layout.size() == 0 {
            return Ok(std::ptr::NonNull::<[u8; 0]>::dangling());
        }

        let padding = self.inner.align_offset(layout.align());
        assert!(
            self.cur.get() + padding + layout.size() <= self.capacity,
            "OOM in arena"
        );
        self.cur.replace(self.cur.get() + padding);

        let s = unsafe {
            std::slice::from_raw_parts_mut(self.inner.add(self.cur.get()), layout.size())
        };
        self.cur.replace(self.cur.get() + layout.size());

        Ok(std::ptr::NonNull::new(s).unwrap())
    }

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: Layout) {}
}

#[derive(Clone)]
pub struct Arena<'a>(&'a ArenaInner);

impl<'a> Arena<'a> {
    pub fn new(inner: &'a ArenaInner) -> Self {
        Self(inner)
    }
}

impl Deref for Arena<'_> {
    type Target = ArenaInner;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
