use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

// implied by UnsafeCell
// impl <T> !Sync for Cell<T>{}
impl<T> Cell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // we know that no one else is concurrently mutating self.value because
        // Cell implements !Sync
        // AND we never invalidate references because we don't give any out
        // we just copy
        unsafe { *self.value.get() = value }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        // we know no one else is mutating since only this thread can mutate
        // because it's !Sync, it is executing this function only
        unsafe { self.value.get().read() }
    }
}

/* Notes to self
 *
 * Cells are useful in single threaded applications
 * Cell does not implement the Sync trait
 *
 * We can have multiple shared references to something
 * Only one thread has a pointer to the cell
 * If we have a shared reference to the cell, no one has a shared reference
 * to the value inside the cell, so it is always safe to update the value
 * inside the cell. When we want to read the value, it is copied.
 * Applications: multiple references to something inside a data structure
 *
 * This something should implement the copy trait, since getting the value
 * copies it
 */

#[cfg(test)]
mod test {
    use super::*;

    // should not compile

    // implied by UnsafeCell
    //impl <T> !Sync for Cell<T>{}
    //#[test]
    //fn bad() {
    //    use std::sync::Arc;
    //    use std::thread;

    //    let x = Arc::new(Cell::new(42));
    //    let x1 = Arc::clone(&x);
    //    thread::spawn(move || x.set(43));
    //    let x2 = Arc::clone(&x);
    //    thread::spawn(move || x.set(44));
    //}

    // this is why our get method returns a copy
    //#[test]
    //fn bad2() {
    //    let x = Cell::new(String::from("hello"));
    //    let first = x.get();
    //    x.set(String::new());
    //    x.set(String::from("world"));
    //    eprint!("{}", first);
    //}
}
