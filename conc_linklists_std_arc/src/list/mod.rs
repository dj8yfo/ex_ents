use std::sync::Arc;

use crate::cell::Cell;

use std::fmt::Debug;
use anyhow::Result;

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
    fn first(&self) -> Result<cursor::Cursor<T>>{
        let pre_cell = self.first.clone();
        let pre_aux = self.first.next_dup().unwrap();
    
        let mut c = cursor::Cursor::new(pre_cell, pre_aux);
    
        c.update()?;
        Ok(c)
    }

    
}
#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super:: List;
    use anyhow::Result;

    #[test]
    fn test_new() {
        let list: List<u32> = List::new();

        let cursor = list.first().unwrap();

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


        let mut cursor = list.first().unwrap();

        cursor.try_insert(42).unwrap();

        assert!(cursor.try_insert(42).is_err());

        cursor.update().unwrap();

        cursor.try_insert(84).unwrap();
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
        let mut cursor = list.first().unwrap();

        for _ in 0..ITER {

            cursor.try_insert(42).unwrap();
            cursor.update().unwrap();
        }
        let mut count = 0;
        while cursor.next().unwrap() {
            count += 1;
        }
        assert_eq!(count, ITER);

    }

    #[test]
    fn test_next_complex_parallel() {
        let list: Arc<List<u32>> = Arc::new(List::new());

        let mut vec_jh = vec![];
        const NUM_THREADS: usize = 1000;

        for _ in 0..NUM_THREADS {
            let list_copy = Arc::clone(&list);
            let jh = thread::spawn(move || -> Result<()> {
                let mut cursor = list_copy.first()?;

                for _ in 0..ITER {
                    cursor.insert(42)?;
                }
                Ok(())
            });
            vec_jh.push(jh);
        }

        for jh in vec_jh {
            jh.join().unwrap().unwrap();
        }

        let mut cursor = list.first().unwrap();
        let mut count = 0;
        while cursor.next().unwrap() {
            count += 1;
        }
        assert_eq!(count, ITER*NUM_THREADS);

    }

    #[test]
    fn test_set_backlink() {

        let list: List<u32> = List::new();

        let mut cursor = list.first().unwrap();

        cursor.try_insert(42).unwrap();
        cursor.update().unwrap();
        cursor.target.as_ref().unwrap().store_backlink(
            Arc::downgrade(&list.first.clone()) 
        );

        let backlink = cursor.target.as_ref().unwrap().backlink();

        assert_eq!(
            Arc::as_ptr(&backlink.unwrap()),
            Arc::as_ptr(&list.first)
        );

        drop(cursor);
    }



}
