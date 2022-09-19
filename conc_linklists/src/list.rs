use std::sync::atomic::AtomicPtr;

use super::cell::{Cell, Dummy};

pub struct List<T> {
    first: *const Cell<T>,
    last: *const Cell<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        let last_box = Box::new(Cell::Dummy(Dummy::Last));
        let last_ptr = Box::into_raw(last_box);

        let aux_box = Box::new(Cell::aux(1, last_ptr));
        let first_box = Box::new(Cell::first(1, Box::into_raw(aux_box)));

        let first_ptr = Box::into_raw(first_box);

        List {
            first: first_ptr,
            last: last_ptr,
        }
    }
}

#[cfg(test)]
mod tests {}
