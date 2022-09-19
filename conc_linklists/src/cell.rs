use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

pub static LAST_VAR_MESSAGE: &str = "not expecting last cell variant here";

pub struct Links<T> {
    next: AtomicPtr<Cell<T>>,
    back_link: Option<AtomicPtr<Cell<T>>>,
    ref_counter: AtomicUsize,
    claimed: AtomicBool,
}

pub enum Dummy<T> {
    First(Links<T>),
    Last,
}

pub enum Cell<T> {
    Data { links: Links<T>, data: T },
    Aux { links: Links<T> },
    Dummy(Dummy<T>),
}

const GREATER_THAN_ONE: usize = 10;

pub fn safe_read_ptr<T>(q: *const Cell<T>) -> *const Cell<T> {
    if q.is_null() {
        panic!("null pointer value of atomic pointer!");
    }
    match unsafe { &*q } {
        Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
            links.ref_counter.fetch_add(1, Ordering::Release);
        }
        Dummy(Last) => {}
        // TODO: add refcount to Last for drop impl
    };
    q
}
pub fn safe_read<T>(p: &AtomicPtr<Cell<T>>) -> *const Cell<T> {
    loop {
        let q = p.load(Ordering::Acquire);
        safe_read_ptr(q);
        if q == p.load(Ordering::Acquire) {
            return q;
        } else {
            release(q);
        }
    }
}

pub fn release_opt<T>(p: Option<*mut Cell<T>>) {
    match p {
        None => {}
        Some(p) => release(p),
    }
}
pub fn release<T>(p: *mut Cell<T>) {
    let cnt = match unsafe { &*p } {
        Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
            links.ref_counter.fetch_sub(1, Ordering::AcqRel)
        }
        // TODO: add refcount to Last for drop impl
        Dummy(Last) => GREATER_THAN_ONE,
    };
    if cnt > 1 {
        return;
    }
    let claimed = match unsafe { &*p } {
        Data { ref links, .. } | Aux { ref links }  => {
            links.claimed.fetch_or(true, Ordering::AcqRel)
        }
        Dummy(Last) | Dummy(First(..)) => true,
    };
    if claimed {
        return;
    } else {
        reclaim(p);
    }
}

fn reclaim<T>(p: *mut Cell<T>) {
    let mut p_box: Box<Cell<T>> = unsafe { Box::from_raw(p) };
}

use Cell::*;
use Dummy::*;

impl<T> Cell<T> {
    pub fn aux(ref_counter: usize, next: *mut Cell<T>) -> Cell<T> {
        Aux {
            links: Links {
                next: AtomicPtr::new(next),
                back_link: None,
                ref_counter: AtomicUsize::new(ref_counter),
                claimed: AtomicBool::new(false),
            },
        }
    }

    pub fn first(ref_counter: usize, next: *mut Cell<T>) -> Cell<T> {
        Dummy(First(Links {
            next: AtomicPtr::new(next),
            back_link: None,
            ref_counter: AtomicUsize::new(ref_counter),
            claimed: AtomicBool::new(false),
        }))
    }

    pub fn next(&self) -> Option<&AtomicPtr<Cell<T>>> {
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                Some(&links.next)
            }
            Dummy(Last) => None,
        }
    }

    pub fn set_next(&self, next: *mut Cell<T>) -> bool {
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                links.next.store(next, Ordering::Release);
                true
            }
            Dummy(Last) => false,
        }
    }
    pub fn is_data_cell(&self) -> bool {
        match self {
            Data { .. } => true,
            Aux { .. } => false,
            Dummy(First(..)) => false,
            Dummy(Last) => false,
        }
    }
    pub fn is_aux(&self) -> bool {
        match self {
            Data { .. } => false,
            Aux { .. } => true,
            Dummy(First(..)) => false,
            Dummy(Last) => false,
        }
    }
    pub fn is_last(&self) -> bool {
        match self {
            Data { .. } => false,
            Aux { .. } => false,
            Dummy(First(..)) => false,
            Dummy(Last) => true,
        }
    }
    pub fn is_after_aux(&self) -> bool {
        match self {
            Data { .. } => true,
            Aux { .. } => false,
            Dummy(First(..)) => false,
            Dummy(Last) => true,
        }
    }
}
