use std::ops::{Deref, DerefMut};
use crate::loom::cell::CausalCell;
use crate::loom::sync::atomic::{AtomicBool, Ordering};


pub struct Mutex<T> {
    flag: AtomicBool,
    cell: CausalCell<T>
}

pub struct MutexGuard<'a, T: 'a> {
    inner: &'a Mutex<T>
}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Mutex<T> {
        Mutex {
            flag: AtomicBool::new(false),
            cell: CausalCell::new(t)
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if !self.flag.swap(true, Ordering::Acquire) {
            Some(MutexGuard { inner: self })
        } else {
            None
        }
    }
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.cell.with(|ptr| unsafe { &*ptr })
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.cell.with_mut(|ptr| unsafe { &mut *ptr })
    }
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.inner.flag.store(false, Ordering::Release);
    }
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}
