use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::cell::{
    release, safe_read, safe_read_ptr, Cell, Dummy, LAST_VAR_MESSAGE
};

mod cursor;

use cursor::Cursor;

pub struct List<T> {
    first: *const Cell<T>,
    last: *const Cell<T>,
}

unsafe impl<T> Send for List<T> {}
unsafe impl<T> Sync for List<T> {}

use std::fmt::Debug;
impl<T: Debug> List<T> {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn first(&self, c: &mut Cursor<T>) {
        c.pre_cell = safe_read_ptr(self.first) as *mut Cell<T>;
        let first_next = unsafe { (*self.first).next().expect(LAST_VAR_MESSAGE) };
        c.pre_aux = safe_read(first_next) as *mut Cell<T>;
        c.target = None;

        c.update(self.last as *mut Cell<T>);
    }

    #[allow(dead_code)]
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

        debug_assert!({
            unsafe {
                println!(
                    "[run csw on cursor.pre_aux.next()]:  {:?} {:p} -> {:?} {:p}",
                    cursor_target.as_ref(),
                    cursor_target,
                    inserted.data.as_ref(),
                    inserted.data
                );
            }
            true
        });
        cursor_pre_aux_next
            .compare_exchange(
                cursor_target,
                inserted.data,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    #[allow(dead_code)]
    fn try_delete(&self, c: &mut Cursor<T>) -> Option<bool> {
        let d: *mut Cell<T> = match c.get_target_not_last(self.last as *mut Cell<T>) {
            Ok(ptr) => ptr,
            Err(opt) => return opt,
        };
        let n =
            unsafe { safe_read((*d).next().expect(LAST_VAR_MESSAGE)) as *mut Cell<T> };
        let pre_aux_next = unsafe { (*c.pre_aux).next().expect(LAST_VAR_MESSAGE) };

        debug_assert!({
            unsafe {
                println!(
                    "[run csw on cursor.pre_aux.next()]:  {:?} {:p} -> {:?} {:p}",
                    d.as_ref(),
                    d,
                    n.as_ref(),
                    n
                );
            }
            true
        });
        let r = pre_aux_next.compare_exchange(d, n, Ordering::AcqRel, Ordering::Acquire);
        if r.is_err() {
            return Some(false);
        }
        release(d);
        self.set_and_cycle_backlink(c, d, n)
    }

    fn set_and_cycle_backlink(
        &self,
        c: &mut Cursor<T>,
        d: *mut Cell<T>, // deleted target
        n: *mut Cell<T>, // aux after target
    ) -> Option<bool> {
        assert!(unsafe { (*d).set_backlink(c.pre_cell) });
        let mut p = safe_read_ptr(c.pre_cell) as *mut Cell<T>;

        loop {
            let p_back_link = unsafe { (*p).backlink().expect(LAST_VAR_MESSAGE) };
            if p_back_link.load(Ordering::Acquire).is_null() {
                break;
            }
            let q = safe_read(p_back_link);
            release(p);
            p = q as *mut Cell<T>;
        }

        let s = safe_read(unsafe { (*p).next().expect(LAST_VAR_MESSAGE) });

        self.advance_n_to_rightmost_aux(p, s as *mut Cell<T>, n)
    }

    fn advance_n_to_rightmost_aux(
        &self,
        p: *mut Cell<T>,     // firstmost non-null backlink
        s: *mut Cell<T>,     // p's next
        mut n: *mut Cell<T>, // aux after target
    ) -> Option<bool> {
        loop {
            let n_next = unsafe { (*n).next().expect(LAST_VAR_MESSAGE) };
            let cond = unsafe { (*n_next.load(Ordering::Acquire)).is_after_aux() };
            if cond {
                break;
            }
            let q = safe_read(n_next);
            release(n);
            n = q as *mut Cell<T>;
        }

        self.delete_csw_chain(p, s, n)
    }

    fn delete_csw_chain(
        &self,
        p: *mut Cell<T>,     // firstmost non-null backlink
        mut s: *mut Cell<T>, // p's next
        n: *mut Cell<T>,     // aux after target
    ) -> Option<bool> {
        loop {
            let p_next = unsafe { (*p).next().unwrap() };

            debug_assert!({
                unsafe {
                    println!(
                        "[run csw on firstmost_backlink.next()] {:?} {:p} -> {:?} {:p}",
                        s.as_ref(),
                        s,
                        n.as_ref(),
                        n
                    );
                }
                true
            });
            let r = p_next.compare_exchange(s, n, Ordering::AcqRel, Ordering::Acquire);
            release(s);
            if r.is_err() {
                s = safe_read(p_next) as *mut Cell<T>;
            }
            if List::delete_break_cond(r.is_ok(), p, n) {
                break;
            }
        }
        release(p);
        release(s);
        release(n);
        Some(true)
    }
    fn delete_break_cond(result: bool, p: *mut Cell<T>, n: *mut Cell<T>) -> bool {
        let back_not_null = !unsafe {
            (*p).backlink()
                .expect(LAST_VAR_MESSAGE)
                .load(Ordering::Acquire)
                .is_null()
        };
        let n_next =
            unsafe { (*n).next().expect(LAST_VAR_MESSAGE).load(Ordering::Acquire) };
        let n_next_not_normal =
            unsafe { !n_next.as_ref().expect("not null").is_after_aux() };

        result || back_not_null || n_next_not_normal
    }

    #[allow(dead_code)]
    fn next(&self, c: &mut Cursor<T>) -> Option<bool> {
        let target_ptr = match c.get_target_not_last(self.last as *mut Cell<T>) {
            Ok(ptr) => ptr,
            Err(opt) => return opt,
        };
        release(c.pre_cell);
        c.pre_cell = safe_read_ptr(target_ptr) as *mut Cell<T>;
        release(c.pre_aux);
        let c_target_next = unsafe { (*target_ptr).next().expect(LAST_VAR_MESSAGE) };
        c.pre_aux = safe_read(c_target_next) as *mut Cell<T>;
        c.update(self.last as *mut Cell<T>);
        Some(true)
    }

    #[allow(dead_code)]
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

        drop(cursor);
    }

    #[allow(clippy::clone_on_copy)]
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
        drop(cursor);

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
        drop(cursor);

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
    fn test_try_delete() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);

        list.insert(&mut cursor, 42);
        list.insert(&mut cursor, 84);

        cursor.update(list.last as *mut Cell<u32>);

        unsafe {
            let mut cnt = 0;
            let mut p = list.first;
            while !p.as_ref().unwrap().is_last() {
                println!("{} {:?} {:p}", cnt, p.as_ref(), p);
                cnt += 1;
                p = p.as_ref().unwrap().next().unwrap().load(Ordering::Acquire);
            }
            println!("{} {:?} {:p}", cnt, p.as_ref(), p);

        }
        let mut r = list.try_delete(&mut cursor);
        assert_eq!(r, Some(true));


        unsafe {
            let mut cnt = 0;
            let mut p = list.first;
            while !p.as_ref().unwrap().is_last() {
                println!("{} {:?} {:p}", cnt, p.as_ref(), p);
                cnt += 1;
                p = p.as_ref().unwrap().next().unwrap().load(Ordering::Acquire);
            }
            println!("{} {:?} {:p}", cnt, p.as_ref(), p);
            assert_eq!(cnt, 4);

        }

        unsafe {
            let f_aux = (*list.first).next().unwrap().load(Ordering::Relaxed);
            let f_val = (*f_aux).next().unwrap().load(Ordering::Relaxed);
        
            assert_eq!((*f_val).val(), Some(&42));
        
            let s_aux = (*f_val).next().unwrap().load(Ordering::Relaxed);
            let s_val = (*s_aux).next().unwrap().load(Ordering::Relaxed);
        
            assert!(s_val.as_ref().unwrap().is_last());
        }
        r = list.try_delete(&mut cursor);
        assert_eq!(r, Some(false));
        r = list.try_delete(&mut cursor);
        assert_eq!(r, Some(false));

        assert!(list.next(&mut cursor).is_some());
        r = list.try_delete(&mut cursor);
        assert_eq!(r, Some(true));
        r = list.try_delete(&mut cursor);
        assert_eq!(r, Some(false));

        assert!(list.next(&mut cursor).is_some());
        r = list.try_delete(&mut cursor);

        // last position
        assert_eq!(r, None);
        assert!(list.next(&mut cursor).is_none());

        unsafe {
            let mut cnt = 0;
            let mut p = list.first;
            while !p.as_ref().unwrap().is_last() {
                println!("{} {:?} {:p}", cnt, p.as_ref(), p);
                cnt += 1;
                p = p.as_ref().unwrap().next().unwrap().load(Ordering::Acquire);
            }
            println!("{} {:?} {:p}", cnt, p.as_ref(), p);
            assert_eq!(cnt, 2);

        }

        unsafe {
            let f_aux = (*list.first).next().unwrap().load(Ordering::Relaxed);
            let f_val = (*f_aux).next().unwrap().load(Ordering::Relaxed);
        
            assert!(f_val.as_ref().unwrap().is_last());
        }
        drop(cursor);

    }


    const ITER: usize = 1000;

    #[test]
    fn test_next() {
        let list: List<u32> = List::new();

        let mut cursor = Cursor::empty();

        list.first(&mut cursor);

        
        for _ in 0..ITER {
            list.insert(&mut cursor, 42);
        }

        let mut count = 0;
        while list.next(&mut cursor).is_some() {
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
            let jh = thread::spawn(move || {
                let mut cursor = Cursor::empty();

                list_copy.first(&mut cursor);

                for _ in 0..ITER {
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
        while list.next(&mut cursor).is_some() {
            count += 1;
        }
        assert_eq!(count, ITER*NUM_THREADS);

        cursor = Cursor::empty();

        list.first(&mut cursor);
        let mut count = 0;
        while list.next(&mut cursor).is_some() {
            count += 1;
        }
        assert_eq!(count, ITER*NUM_THREADS);
    }
}
