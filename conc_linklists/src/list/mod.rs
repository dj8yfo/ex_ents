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

    fn try_insert(c: &mut Cursor<T>, data: *mut Cell<T>, aux: *mut Cell<T>) -> bool {
        let cursor_pre_aux_next: &AtomicPtr<Cell<T>>;
        let cursor_target: *mut Cell<T>;
        unsafe {
            assert!((*data).is_data_cell());
            assert!((*aux).is_aux());

            assert!((*data).set_next(aux));
            cursor_target = c.target.expect("target in cursor should be initialized");
            assert!((*aux).set_next(cursor_target));

            cursor_pre_aux_next = (*c.pre_aux).next().expect(LAST_VAR_MESSAGE);
        }
        cursor_pre_aux_next
            .compare_exchange(cursor_target, data, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    // fn prep_val(val: T) -> Inserted<T> {
    //     let aux_box = Box::new(Cell::aux(1, ptr::null_mut()));
    //     Inserted {
    //         aux: Box::into_raw(aux_box),
    //     }
    //    
    // }
}

struct Inserted<T>{
    data: *mut Cell<T>,
    aux: *mut Cell<T>,
}

#[cfg(test)]
mod tests {
    use super::{List, cursor::Cursor};

    #[test]
    fn test_new() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);
    }
}
