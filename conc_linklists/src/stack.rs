use std::sync::atomic::AtomicPtr;


use super::cell::{Dummy, Cell};


pub struct List<T> {
    first: AtomicPtr<Cell<T>>,
    last: AtomicPtr<Cell<T>>,
}

impl<T> List<T> {
    fn new() -> Box<Cell<T>> {
        let last_box = Box::new(Cell::Dummy(Dummy::Last));

        
        let aux_box = Box::new(Cell::aux(Box::into_raw(last_box)));
        aux_box
    }
}
