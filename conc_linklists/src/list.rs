use std::sync::atomic::AtomicPtr;

use super::cell::{Cell, Dummy};

pub struct List<T> {
    first: AtomicPtr<Cell<T>>,
    last: AtomicPtr<Cell<T>>,
}

impl<T> List<T> {
    fn new() -> Self {
        let last_box = Box::new(Cell::Dummy(Dummy::Last));
        let last_ptr = Box::into_raw(last_box);

        let aux_box = Box::new(Cell::aux(1, last_ptr));
        let first_box = Box::new(Cell::first(1, Box::into_raw(aux_box)));

        let first_ptr = Box::into_raw(first_box);

        List {
            first: AtomicPtr::new(first_ptr),
            last: AtomicPtr::new(last_ptr),
        }
    }
}

#[cfg(test)]
mod tests {}
