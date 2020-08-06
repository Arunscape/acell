// rc is a single treaded reference counted pointer
// provides shared ownership to a value on the heap'
// Clone will make a new pointer which points to the same value on the heap
//
// similar to refcell in the sense that it counts references
// dissimilar in that it does not provide mutability
//
// useful for datastructures like graphs
//
// say you have a large string or blob in a program, you just want to keep pointers to it
// you want to de-allocate it only when all references to it are gone
// !Sync + !Send
// not thread safe
//
use crate::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

struct RcInner<T> {
    value: T,
    refcount: Cell<usize>,
}

// Rust does no know that this type owns a T
// it knows this type has a pointer to a T
// when the Rc goes away, it doesn't know that there might be a T that gets dropped
// this matters if T contains a lifetime
//
// when Rust does its drop check
// imagine you have a type that contains a reference
// when it's dropped, it modifies that reference
// struct Foo<'a, T> {
//     v: &mut T,
// }
// impl<T> Drop for Foo<'_, T>{
// fn drop(&mut self) {
//    self.v.mutablefunction();
// }
// }
// fn main{
//     let t = String::from("hello");
//     let foo = Foo{ v: &mut t};
//     drop(t);
//     // WAIT t is already dropped
//     drop(foo);
// }
//
//
// the marker makes sure Rust knows to check if T is dropped
// it lets the compiler know we own T

// in the real std lib, Rc supports T: ?Sized i.e. T can be unsized
pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::new(RcInner {
            value,
            refcount: Cell::new(1),
        });
        Rc {
            // Box does not give us a null pointer
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        inner.refcount.set(inner.refcount.get() + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // self.inner is a Box that is only deallocated when the last rc goes away
        &unsafe { self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let count = inner.refcount.get();
        match count {
            // no more references to the inner value
            1 => {
                drop(inner);
                drop(unsafe { Box::from_raw(self.inner.as_ptr()) })
            }
            _ => inner.refcount.set(count - 1),
        }
    }
}
