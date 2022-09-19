use std::{sync::atomic::{AtomicPtr, Ordering}};
use crate::cell::{release_opt, release};

use super::cell::{Cell, safe_read};

pub struct Cursor<T> {
    target: Option<*mut Cell<T>>,
    pre_aux: *mut Cell<T>,
    pre_cell: *mut Cell<T>,

}

fn cmp<T>(a: Option<&AtomicPtr<Cell<T>>>, b: Option<*mut Cell<T>>) -> bool {
    let equal = a.map_or_else(
        || b.is_none(),
        |ptr| {
            let val: *mut Cell<T> = ptr.load(Ordering::Acquire);
            match b {
                None => false,
                Some(target) => val == target,
            }
        },
    );
    equal
}

static LAST_VAR_MESSAGE : &str = "not expecting last cell variant here";

impl<T> Cursor<T> {
    fn update(&mut self, last: *mut Cell<T>) {
        let pre_aux_next = unsafe { (*(self.pre_aux)).next() };
        let equal = cmp(pre_aux_next, self.target);
        if equal {
            return;
        }

        let mut p = self.pre_aux; // expecting aux variant
        let mut n = safe_read(unsafe { (*p).next().expect(LAST_VAR_MESSAGE) });
        release_opt(self.target);
        loop {
            let cond = (n != last) && unsafe { !(*n).is_data_cell() };
            if !cond {
                break;
            }

            let pre_cell_next =
                unsafe { (*self.pre_cell).next().expect(LAST_VAR_MESSAGE) };
            assert!(pre_cell_next
                .compare_exchange(
                    p,
                    n as *mut Cell<T>,
                    Ordering::AcqRel,
                    Ordering::Acquire
                )
                .is_ok());
            release(p);
            p = n as *mut Cell<T>;
            n = safe_read(unsafe { (*p).next().expect(LAST_VAR_MESSAGE) });
        }
        self.pre_aux = p;
        self.target = Some(n as *mut Cell<T>);
    }
}
