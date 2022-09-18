use std::{sync::atomic::{AtomicPtr, Ordering}};
use crate::cell::release_opt;

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

impl<T> Cursor<T> {
    fn update(&mut self, last: *mut Cell<T>) {
        let pre_aux_next = unsafe { (*(self.pre_aux)).next() };
        let equal = cmp(pre_aux_next, self.target);
        if equal {
            return;
        }

        let p = self.pre_aux; // expecting aux variant
        // TODO: assert variant IS_AUX
        let n = safe_read(unsafe {
            (*p).next().expect("not expecting last cell variant here")
        });
        release_opt(self.target);
    }
}
