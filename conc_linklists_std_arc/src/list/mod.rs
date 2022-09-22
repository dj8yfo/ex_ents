use std::sync::Arc;

use crate::cell::{Cell};

use std::fmt::Debug;

mod cursor;

pub struct List<T: Debug> {
    first: Arc<Cell<T>>,
    last: Arc<Cell<T>>,
}

impl<T: Debug> List<T> {
    #[allow(dead_code)]
    fn new() -> Self {
        let last = Cell::new_last();
        let last_clone = last.clone();

        let aux = Cell::new_aux(last_clone);

        List {
            first: Cell::new_first(aux),
            last,
        }
    }

    #[allow(dead_code)]
    fn first(&self) -> cursor::Cursor<T>{
        let pre_cell = self.first.clone();
        let pre_aux = self.first.next_dup().unwrap();
    
        let mut c = cursor::Cursor::new(pre_cell, pre_aux);
    
        c.update();
        c
    }

    
}
