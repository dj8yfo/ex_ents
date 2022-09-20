use std::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::cell::{
    release, safe_read, safe_read_ptr, Cell, Dummy, LAST_VAR_MESSAGE,
    TARGET_NULL_MESSAGE,
};

mod cursor;

use cursor::Cursor;

pub struct List<T> {
    first: *const Cell<T>,
    last: *const Cell<T>,
}

unsafe impl<T> Send for List<T> {}
unsafe impl<T> Sync for List<T> {}

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

    fn try_delete(&self, c: &mut Cursor<T>) -> Option<bool> {
        let d: *mut Cell<T>;
        match c.target {
            None => { return Some(false)}
            Some(target) => {
                if target == self.last as *mut Cell<T> {
                    return None;
                }
                d = target;
            }
        }
        Some(true)
    }


    fn next(&self, c: &mut Cursor<T>) -> Option<bool> {
        let target_ptr: *mut Cell<T>;
        match c.target {
            None => { return Some(false)}
            Some(target) => {
                if target == self.last as *mut Cell<T> {
                    return None;
                }
                target_ptr = target;
            }
        }
        release(c.pre_cell);
        c.pre_cell = safe_read_ptr(target_ptr) as *mut Cell<T>;
        release(c.pre_aux);
        let c_target_next = unsafe {(*target_ptr).next().expect(LAST_VAR_MESSAGE)};
        c.pre_aux = safe_read(c_target_next) as *mut Cell<T>;
        c.update(self.last as *mut Cell<T>);
        Some(true)
    }
    fn insert(&self, c: &mut Cursor<T>, val: T) {
        let inserted = List::prep_val(val);
        loop {
            let res = List::try_insert(c, inserted);
            if res {
                c.update(self.last as *mut Cell<T>);
                break;
            }

            c.update(self.last as *mut Cell<T>);
        }
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

struct Inserted<T> {
    data: *mut Cell<T>,
    aux: *mut Cell<T>,
}

impl<T> Copy for Inserted<T> {}
impl<T> Clone for Inserted<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            aux: self.aux,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::{atomic::Ordering, Arc}, thread};

    use crate::cell::Cell;

    use super::{cursor::Cursor, List};

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

    #[test]
    fn test_insert() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);

        list.insert(&mut cursor, 42);
        list.insert(&mut cursor, 84);

        unsafe {
            let f_aux = (*list.first).next().unwrap().load(Ordering::Relaxed);
            let f_val = (*f_aux).next().unwrap().load(Ordering::Relaxed);

            assert_eq!((*f_val).val(), Some(&84));

            let s_aux = (*f_val).next().unwrap().load(Ordering::Relaxed);
            let s_val = (*s_aux).next().unwrap().load(Ordering::Relaxed);

            assert_eq!((*s_val).val(), Some(&42));
        }
    }

    const ITER: usize = 1000;

    #[test]
    fn test_next() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);

        
        for i in 0..ITER {
            list.insert(&mut cursor, 42);
        }

        let mut count = 0;
        while let Some(res) = list.next(&mut cursor) {
            if res {
                count += 1;
            } else {
                break;
            }
        }
        assert_eq!(count, ITER);

    }

    #[test]
    fn test_next_complex_parallel() {
        let list: Arc<List<u32>> = Arc::new(List::new());

        let mut vec_jh = vec![];
        const NUM_THREADS: usize = 1000;

        for i in 0..NUM_THREADS {
            let list_copy = Arc::clone(&list);
            let jh = thread::spawn(move || {
                let mut cursor = Cursor::empty();

                list_copy.first(&mut cursor);

                for i in 0..ITER {
                    list_copy.insert(&mut cursor, 42);
                }
            });
            vec_jh.push(jh);
        }

        for jh in vec_jh {
            jh.join().unwrap();
        }

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);
        let mut count = 0;
        while let Some(_) = list.next(&mut cursor) {
            count += 1;
        }
        assert_eq!(count, ITER*NUM_THREADS);
    }
}
