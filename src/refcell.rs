use crate::cell::Cell;
use std::cell::UnsafeCell;

#[derive(Copy, Clone)]
pub enum RefState {
    Unshared,
    Shared(usize),
    Exclusive,
}

pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<RefState>,
}

pub struct Ref<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Shared(1) => self.refcell.state.set(RefState::Unshared),
            RefState::Shared(n) => self.refcell.state.set(RefState::Shared(n - 1)),
            // it's a shared reference
            _ => unreachable!(),
        }
    }
}

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // a Ref is only created if no exclusive references have been given out
        // when it's given out, it's shared, so no exclusive references are
        // given out
        //
        // so dereferencing into a shared reference is fine
        unsafe { &*self.refcell.value.get() }
    }
}

pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.state.get() {
            RefState::Exclusive => self.refcell.state.set(RefState::Unshared),
            _ => unreachable!(),
        }
    }
}

impl<T> std::ops::Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // see safety for DerefMut
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> std::ops::DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // a RefMut  is only created if no references were given out
        // when it's given out, it's in the exclusive state
        // so no future references are given out
        // so we have exclusive access to the inner value
        // so mutably dereferencing is fine
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> RefCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(RefState::Unshared),
        }
    }

    /// return Some(&value) if no exclusive reference (mutable) was given out
    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        // no exclusive references given out since state would be exclusive
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Shared(1));
                Some(Ref { refcell: self })
            }
            RefState::Shared(n) => {
                self.state.set(RefState::Shared(n + 1));
                Some(Ref { refcell: self })
            }
            RefState::Exclusive => None,
        }
    }

    /// if you try to exclusively borrow but it already has been, you get None
    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        // no other references given out since state would be shared(_) or exclusive
        match self.state.get() {
            RefState::Unshared => {
                self.state.set(RefState::Exclusive);
                Some(RefMut { refcell: self })
            }
            _ => None,
        }
    }
}

/* Notes
 * Refcells let you check at runtime if someone else is mutating the value
 * contained within
 *
 * useful for traversing a graph with cycles, and trees
 * for example if you're recursing, you already have a mutable reference, and
 * later on you're trying to get a mutable reference to the same thing again
 *
 * safe dynamically checked borrowing
 */
