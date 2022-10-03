use std::marker::PhantomData;

use appkit_nsworkspace_bindings::{
    id,
    INSArray,
    NSArray as InnerNSArray,
};

pub struct NSArray<T: 'static> {
    inner: InnerNSArray,
    phantom: PhantomData<T>,
}

impl<T: 'static> NSArray<T> {
    pub fn iter(&self) -> NSArrayIter<'_, T> {
        NSArrayIter {
            inner: &self.inner,
            count: self.len(),
            index: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        unsafe { <InnerNSArray as INSArray<T>>::count(&self.inner) }
    }
}

impl<T: 'static> From<InnerNSArray> for NSArray<T> {
    fn from(arr: InnerNSArray) -> NSArray<T> {
        NSArray {
            inner: arr,
            phantom: PhantomData,
        }
    }
}

pub struct NSArrayIter<'a, T: 'static> {
    inner: &'a InnerNSArray,
    count: u64,
    index: u64,
    phantom: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for NSArrayIter<'a, T> {
    type Item = id;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let item = unsafe { <InnerNSArray as INSArray<T>>::objectAtIndex_(self.inner, self.index) };
            self.index += 1;
            Some(item)
        }
    }
}

impl<'a, T> IntoIterator for &'a NSArray<T> {
    type IntoIter = NSArrayIter<'a, T>;
    type Item = id;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
