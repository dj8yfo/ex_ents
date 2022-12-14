use std::sync::atomic::{AtomicPtr, Ordering};
use crate::cell::{release_opt, release, LAST_VAR_MESSAGE, ReclaimCnt};

use crate::cell::{Cell, safe_read};
use std::fmt::Debug;

pub struct Cursor<'a, T: Debug> {
    pub(super) reclaim: &'a mut ReclaimCnt,
    pub(super) target: Option<*mut Cell<T>>,
    pub(super) pre_aux: *mut Cell<T>,
    pub(super) pre_cell: *mut Cell<T>,

}
impl<'a, T: Debug> Drop for Cursor<'a, T> {
    fn drop(&mut self) {
        release_opt(self.reclaim, self.target);
        release(self.reclaim,self.pre_aux);
        release(self.reclaim,self.pre_cell);
    }
}

fn cmp<T>(a: Option<&AtomicPtr<Cell<T>>>, b: Option<*mut Cell<T>>) -> bool {
    a.map_or_else(
        || b.is_none(),
        |ptr| {
            let val: *mut Cell<T> = ptr.load(Ordering::Acquire);
            match b {
                None => false,
                Some(target) => val == target,
            }
        },
    )
}


impl<'a, T: Debug> Cursor<'a, T> {
    #[allow(dead_code)]
    pub fn empty(reclaim: &'a mut ReclaimCnt) -> Self {
        Self {
            reclaim,
            target: None,
            pre_aux: std::ptr::null_mut(),
            pre_cell: std::ptr::null_mut(),
        }
    }

    pub fn update(&mut self, last: *mut Cell<T>) {
        let pre_aux_next = unsafe { (*(self.pre_aux)).next() };
        let equal = cmp(pre_aux_next, self.target);
        if equal {
            return;
        }

        let mut p = self.pre_aux; // expecting aux variant
        let mut n = safe_read(unsafe { (*p).next().expect(LAST_VAR_MESSAGE) });
        release_opt(self.reclaim, self.target);
        loop {
            let cond = (n != last) && unsafe { !(*n).is_after_aux() };
            if !cond {
                break;
            }

            let pre_cell_next =
                unsafe { (*self.pre_cell).next().expect(LAST_VAR_MESSAGE) };

            let _r = pre_cell_next
                .compare_exchange(
                    p,
                    n as *mut Cell<T>,
                    Ordering::AcqRel,
                    Ordering::Acquire
                );
            release(self.reclaim, p);
            p = n as *mut Cell<T>;
            n = safe_read(unsafe { (*p).next().expect(LAST_VAR_MESSAGE) });
        }
        self.pre_aux = p;
        self.target = Some(n as *mut Cell<T>);
    }

    pub fn get_target_not_last(&self, last: *mut Cell<T>) -> Result<*mut Cell<T>, Option<bool>> {
        match self.target {
            None => Err(Some(false)),
            Some(target) => {
                if target == last {
                    return Err(None);
                }
                Ok(target)
            }
        }
    }
}
