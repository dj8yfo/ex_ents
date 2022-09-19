
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

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

pub fn safe_read_ptr<T>(q: *mut Cell<T>) -> *const Cell<T> {
    if q.is_null() {
        panic!("null pointer value of atomic pointer!");
    }
    match unsafe { &*q } {
        Cell::Data { ref links, .. } => {
            links.ref_counter.fetch_add(1, Ordering::Release);
        }
        Cell::Aux { ref links } => {
            links.ref_counter.fetch_add(1, Ordering::Release);
        }
        Cell::Dummy(Dummy::First(ref links)) => {
            links.ref_counter.fetch_add(1, Ordering::Release);
        }
        Cell::Dummy(Dummy::Last) => {}
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
        None => {},
        Some(p) => release(p),
    }
}

pub fn release<T>(p: *mut Cell<T>) {
    let cnt = match unsafe { &*p } {
        Cell::Data { ref links, .. } => {
            links.ref_counter.fetch_sub(1, Ordering::AcqRel)
        }
        Cell::Aux { ref links } => links.ref_counter.fetch_sub(1, Ordering::AcqRel),
        Cell::Dummy(Dummy::First(ref links)) => {
            links.ref_counter.fetch_sub(1, Ordering::AcqRel)
        }
        Cell::Dummy(Dummy::Last) => GREATER_THAN_ONE,
    };
    if cnt > 1 {
        return;
    }
    let claimed = match unsafe { &*p } {
        Cell::Data { ref links, .. } => links.claimed.fetch_or(true, Ordering::AcqRel),
        Cell::Aux { ref links } => links.claimed.fetch_or(true, Ordering::AcqRel),
        Cell::Dummy(Dummy::First(ref links)) => {
            links.claimed.fetch_or(true, Ordering::AcqRel)
        }
        Cell::Dummy(Dummy::Last) => true,
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
impl<T> Cell<T> {
    pub fn aux(ref_counter: usize, next: *mut Cell<T>) -> Cell<T> {
        Cell::Aux {
            links: Links {
                next: AtomicPtr::new(next),
                back_link: None,
                ref_counter: AtomicUsize::new(ref_counter),
                claimed: AtomicBool::new(false),
            },
        }
    }

    pub fn first(ref_counter: usize, next: *mut Cell<T>) -> Cell<T> {
        Cell::Dummy(Dummy::First(
            Links {
                next: AtomicPtr::new(next),
                back_link: None,
                ref_counter: AtomicUsize::new(ref_counter),
                claimed: AtomicBool::new(false),
            },
        ))
    }

    pub fn next(&self) -> Option<&AtomicPtr<Cell<T>>> {
        match self {
            Cell::Data { ref links, .. } => Some(&links.next),
            Cell::Aux { ref links } => Some(&links.next),
            Cell::Dummy(Dummy::First(ref links)) => {
                Some(&links.next)
            }
            Cell::Dummy(Dummy::Last) => None,
        }
    }

    pub fn is_data_cell(&self) -> bool {
        match self {
            Cell::Data {..} => true,
            Cell::Aux {..} => false,
            Cell::Dummy(Dummy::First(..)) => false,
            Cell::Dummy(Dummy::Last) => true,
        }
    }
}

