/// Stack implementation for the Kali VM
pub struct Stack {
    inner: Vec<u8>,
}

impl Stack {
    /// Create a new stack with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Push a value onto the stack.
    unsafe fn push<T>(&mut self, value: T) {
        // stack overflow check
        if std::mem::size_of::<T>() > self.inner.capacity() - self.inner.len() {
            panic!("stack overflow");
        }

        let size = std::mem::size_of::<T>();
        let ptr = &value as *const T as *const u8;

        let dest = self.inner.as_mut_ptr().add(self.inner.len()) as *mut u8;
        std::ptr::copy_nonoverlapping(ptr, dest, size);
        self.inner.set_len(self.inner.len() + size);

        // prevent the value from being dropped
        std::mem::forget(value);
    }

    /// Pop a value from the stack.
    unsafe fn pop<T>(&mut self) -> T {
        let size = std::mem::size_of::<T>();

        // stack underflow check
        if self.inner.len() < size {
            panic!("stack underflow");
        }

        let ptr = self.inner.as_mut_ptr().add(self.inner.len() - size) as *mut T;
        let value = ptr.read();
        self.inner.set_len(self.inner.len() - size);
        value
    }

    /// Push an integer onto the stack.
    pub fn push_int(&mut self, value: i64) {
        unsafe {
            self.push(value);
        }
    }

    /// Pop an integer from the stack.
    pub fn pop_int(&mut self) -> i64 {
        unsafe { self.pop() }
    }

    /// Push a float onto the stack.
    pub fn push_float(&mut self, value: f64) {
        unsafe {
            self.push(value);
        }
    }

    /// Pop a float from the stack.
    pub fn pop_float(&mut self) -> f64 {
        unsafe { self.pop() }
    }

    /// Push a boolean onto the stack.
    pub fn push_bool(&mut self, value: bool) {
        unsafe {
            self.push(value);
        }
    }

    /// Pop a boolean from the stack.
    pub fn pop_bool(&mut self) -> bool {
        unsafe { self.pop() }
    }

    /// Push a string onto the stack.
    pub fn push_str(&mut self, value: &str) {
        unsafe {
            self.push(value);
        }
    }

    /// Pop a string from the stack.
    pub fn pop_str(&mut self) -> &str {
        unsafe { self.pop() }
    }
}
