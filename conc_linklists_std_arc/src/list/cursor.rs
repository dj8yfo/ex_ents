use std::{fmt::Debug, sync::Arc};

use crate::cell::Cell;

pub struct Cursor<T: Debug> {
    target: Option<Arc<Cell<T>>>,
    pre_aux: Arc<Cell<T>>,
    pre_cell: Arc<Cell<T>>,
}


impl<T:Debug> Cursor<T> {
    pub fn new(pre_cell: Arc<Cell<T>>, pre_aux: Arc<Cell<T>>) -> Self {
        Self {
            target: None,
            pre_cell,
            pre_aux,
        }
    }

    pub fn update(&mut self) {
        match self.target {
            None => {},
            Some(ref target) => {
                if self.pre_aux.next_cmp(target) {
                    return
                }
            }
        }

        let mut p = self.pre_aux.clone(); // expecting aux variant
        let mut n = p.next_dup().unwrap();

        drop(self.target.take());
        while !n.is_last() && !n.is_data_cell()  {

            let pre_cell_next = self.pre_cell.next_dup().unwrap();

            pre_cell_next.compare_and_exchange(p, n.clone());

            p = n.clone();
            n = n.next_dup().unwrap();
        }
        self.pre_aux = p;
        self.target = Some(n);
    }


}
