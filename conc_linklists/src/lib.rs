use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

pub struct Links<T> {
    next: AtomicPtr<Cell<T>>,
    back_link: Option<AtomicPtr<Cell<T>>>,
    ref_coutner: AtomicUsize,
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
fn release<T>(p: *mut Cell<T>) {
    let cnt = match unsafe { &*p } {
        Cell::Data { ref links, .. } => {
            links.ref_coutner.fetch_sub(1, Ordering::AcqRel)
        }
        Cell::Aux { ref links } => links.ref_coutner.fetch_sub(1, Ordering::AcqRel),
        Cell::Dummy(Dummy::First(ref links)) => {
            links.ref_coutner.fetch_sub(1, Ordering::AcqRel)
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

#[cfg(test)]
mod tests {}
