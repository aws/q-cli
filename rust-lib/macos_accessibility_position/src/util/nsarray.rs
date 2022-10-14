use std::marker::PhantomData;

use appkit_nsworkspace_bindings::NSArray as AppkitNSArray;
use cocoa::foundation::{
    NSArray as CocoaNSArray,
    NSUInteger,
};

use super::{
    Id,
    IdRef,
};

pub struct NSArray<T: 'static> {
    inner: Id,
    phantom: PhantomData<T>,
}

impl<T: 'static> NSArray<T> {
    pub fn iter(self) -> NSArrayIter<T> {
        let count = self.len();
        NSArrayIter {
            inner: self.inner,
            count,
            index: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        unsafe { self.inner.count() }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> std::ops::Deref for NSArray<T> {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> From<AppkitNSArray> for NSArray<T> {
    fn from(a: AppkitNSArray) -> Self {
        Self {
            inner: unsafe { Id::new(a.0) },
            phantom: PhantomData,
        }
    }
}

pub struct NSArrayIter<T: 'static> {
    inner: Id,
    count: NSUInteger,
    index: NSUInteger,
    phantom: PhantomData<T>,
}

impl<T: 'static> Iterator for NSArrayIter<T> {
    type Item = IdRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let item = unsafe { self.inner.objectAtIndex(self.index) };
            self.index += 1;
            Some(unsafe { IdRef::new(item) })
        }
    }
}

impl<T> IntoIterator for NSArray<T> {
    type IntoIter = NSArrayIter<T>;
    type Item = IdRef;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
