use std::sync::Arc;

use crate::cell::{Cell};

use std::fmt::Debug;

mod cursor;

#[allow(unused)]
pub struct List<T: Debug> {
    first: Arc<Cell<T>>,
    last: Arc<Cell<T>>,
}

impl<T: Debug> Drop for List<T> {
    fn drop(&mut self) {
        let mut cell_it = self.first.next().unwrap();
        while let Some(cell) = cell_it.next() {
            cell_it = cell;
        }
    }

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
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super:: List;

    #[test]
    fn test_new() {
        let list: List<u32> = List::new();

        let cursor = list.first();

        assert_eq!(
            Arc::as_ptr(cursor.target.as_ref().unwrap()),
            Arc::as_ptr(&list.last)
        );

        drop(cursor);
    }

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn test_try_insert() {
        let list: List<u32> = List::new();


        let mut cursor = list.first();

        assert!(cursor.try_insert(42).is_ok());

        assert!(cursor.try_insert(42).is_err());

        cursor.update();

        assert!(cursor.try_insert(84).is_ok());
        drop(cursor);

        let f_aux = (*list.first).next_dup().unwrap();
        let f_val = (*f_aux).next_dup().unwrap();

        assert_eq!((*f_val).val(), Some(&84));

        let s_aux = (*f_val).next_dup().unwrap();
        let s_val = (*s_aux).next_dup().unwrap();

        assert_eq!((*s_val).val(), Some(&42));

    }
    const ITER: usize = 1000;

    #[test]
    fn test_next() {
        let list: List<u32> = List::new();
        let mut cursor = list.first();

        for _ in 0..ITER {

            assert!(cursor.try_insert(42).is_ok());
            cursor.update();
        }

    }

}
