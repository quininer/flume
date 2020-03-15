#![allow(dead_code)]

pub use std::sync;
pub use std::thread;

pub mod cell {
    pub struct CausalCell<T>(std::cell::UnsafeCell<T>);

    impl<T> CausalCell<T> {
        #[inline]
        pub fn new(data: T) -> CausalCell<T> {
            CausalCell(std::cell::UnsafeCell::new(data))
        }

        #[inline]
        pub fn with<F, R>(&self, f: F) -> R
        where
            F: FnOnce(*const T) -> R
        {
            f(self.0.get() as *const _)
        }

        #[inline]
        pub fn with_mut<F, R>(&self, f: F) -> R
        where
            F: FnOnce(*mut T) -> R
        {
            f(self.0.get())
        }
    }
}
