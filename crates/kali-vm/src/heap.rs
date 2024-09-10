use std::alloc::Layout;

pub struct Heap;

impl Heap {
    pub fn new() -> Self {
        Self {}
    }

    /// Allocate a new object on the heap.
    pub fn alloc<T>(&mut self) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        unsafe { std::alloc::alloc(Layout::from_size_align(size, align).unwrap()) as *mut T }
    }

    /// Deallocate an object on the heap.
    pub fn dealloc<T>(&mut self, ptr: *mut T) {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        unsafe {
            std::alloc::dealloc(
                ptr as *mut u8,
                Layout::from_size_align(size, align).unwrap(),
            )
        }
    }
}
