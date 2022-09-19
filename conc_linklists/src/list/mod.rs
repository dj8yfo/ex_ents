use std::{sync::atomic::{AtomicPtr, Ordering}, ptr};

use crate::cell::{safe_read, safe_read_ptr, Cell, Dummy, LAST_VAR_MESSAGE};

mod cursor;

use cursor::Cursor;

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
    fn first(&self, c: &mut Cursor<T>) {
        c.pre_cell = safe_read_ptr(self.first) as *mut Cell<T>;
        let first_next = unsafe { (*self.first).next().expect(LAST_VAR_MESSAGE) };
        c.pre_aux = safe_read(first_next) as *mut Cell<T>;
        c.target = None;

        c.update(self.last as *mut Cell<T>);
    }

    fn try_insert(c: &mut Cursor<T>, inserted: Inserted<T>) -> bool {
        let cursor_pre_aux_next: &AtomicPtr<Cell<T>>;
        let cursor_target: *mut Cell<T>;
        unsafe {
            assert!((*inserted.data).is_data_cell());
            assert!((*inserted.aux).is_aux());

            cursor_target = c.target.expect("target in cursor should be initialized");
            assert!((*inserted.aux).set_next(cursor_target));

            cursor_pre_aux_next = (*c.pre_aux).next().expect(LAST_VAR_MESSAGE);
        }
        cursor_pre_aux_next
            .compare_exchange(
                cursor_target,
                inserted.data,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
    fn prep_val(val: T) -> Inserted<T> {
        let aux_box = Box::new(Cell::aux(1, ptr::null_mut()));
        let aux_ptr = Box::into_raw(aux_box);

        let data_box = Box::new(Cell::data(val, 1, aux_ptr));
        Inserted {
            aux: aux_ptr,
            data: Box::into_raw(data_box),
        }
    }
}

#[derive(Clone)]
struct Inserted<T>{
    data: *mut Cell<T>,
    aux: *mut Cell<T>,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::cell::Cell;

    use super::{List, cursor::Cursor};

    #[test]
    fn test_new() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);
    }
    #[test]
    fn test_try_insert() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);

        let inserted = List::prep_val(42);

        assert!(List::try_insert(&mut cursor, inserted));

        let inserted_fail = List::prep_val(84);
        assert!(!List::try_insert(&mut cursor, inserted_fail.clone()));

        cursor.update(list.last as *mut Cell<u32>);

        assert!(List::try_insert(&mut cursor, inserted_fail));


        unsafe {
            let f_aux = (*list.first).next().unwrap().load(Ordering::Relaxed);
            let f_val = (*f_aux).next().unwrap().load(Ordering::Relaxed);

            assert_eq!((*f_val).val(), Some(&84));

            let s_aux = (*f_val).next().unwrap().load(Ordering::Relaxed);
            let s_val = (*s_aux).next().unwrap().load(Ordering::Relaxed);

            assert_eq!((*s_val).val(), Some(&42));

        }


    }
}
