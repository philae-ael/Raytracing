use std::mem::MaybeUninit;

pub struct Storage<T> {
    inner: Box<[MaybeUninit<T>]>,
    size: usize,
}

impl<T> Storage<T> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Box::new_uninit_slice(size),
            size: 0,
        }
    }

    pub fn store(&mut self, t: T) -> &mut T {
        let ret = self.inner[self.size].write(t);
        self.size += 1;
        ret
    }

    pub fn finish(&self) -> &[T] {
        assert_eq!(self.inner.len(), self.size);
        // SAFETY: this struct preserves the invariant inner[0:size] is initialized
        // Thus thx to the assert, inner is fully initialized
        unsafe { MaybeUninit::slice_assume_init_ref(&self.inner) }
    }
    pub fn finish_mut(&mut self) -> &mut [T] {
        assert_eq!(self.inner.len(), self.size);
        // SAFETY: this struct preserves the invariant inner[0:size] is initialized
        // Thus thx to the assert, inner is fully initialized
        unsafe { MaybeUninit::slice_assume_init_mut(&mut self.inner) }
    }
}
