use std::mem::ManuallyDrop;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicPtr, Arc};

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct Links<T: Debug> {
    next: AtomicPtr<Cell<T>>,
    back_link: AtomicPtr<Cell<T>>,
}

#[derive(Debug)]
pub enum Dummy<T: Debug> {
    First(Links<T>),
    Last,
}

#[derive(Debug)]
pub enum Cell<T: Debug> {
    Data { links: Links<T>, data: T },
    Aux { links: Links<T> },
    Dummy(Dummy<T>),
}

use std::fmt::Debug;


impl<T:Debug> Drop for Cell<T> {
    fn drop(&mut self) {
        debug_assert!({
            println!("dropping {:?}", self);
            true
        })
    }
}

impl<T: Debug> Cell<T> {
    pub fn new_aux(next: Arc<Cell<T>>) -> Arc<Cell<T>> {
        let next = next.conserve();
        use self::Cell::*;
        Arc::new(Aux {
            links: Links {
                next: AtomicPtr::new(next),
                back_link: AtomicPtr::default(),
            },
        })
    }

    pub fn new_data(data: T, next: Arc<Cell<T>>) -> Arc<Cell<T>> {
        let next = next.conserve();
        use self::Cell::*;
        Arc::new(Data {
            data,
            links: Links {
                next: AtomicPtr::new(next),
                back_link: AtomicPtr::default(),
            },
        })
    }

    pub fn new_last() -> Arc<Cell<T>> {
        Arc::new(Cell::Dummy(Dummy::Last))
    }

    pub fn new_first(next: Arc<Cell<T>>) -> Arc<Cell<T>> {
        let next = next.conserve();
        use self::Cell::*;
        use self::Dummy::*;
        Arc::new(Dummy(First(Links {
            next: AtomicPtr::new(next),
            back_link: AtomicPtr::default(),
        })))
    }

    pub fn is_last(&self) -> bool {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { .. } => false,
            Aux { .. } => false,
            Dummy(First(..)) => false,
            Dummy(Last) => true,
        }
    }

    pub fn is_data_cell(&self) -> bool {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { .. } => true,
            Aux { .. } => false,
            Dummy(First(..)) => false,
            Dummy(Last) => false,
        }
    }

    pub fn val(&self) -> Option<&T> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { data , .. }  => {
                Some(data)
            }
            Dummy(Last) | Aux {..} | Dummy(First(..))=> None,
        }
    }


    pub fn conserve(self: Arc<Self>) -> *mut Self {
        Arc::into_raw(self) as *mut Self
    }

    fn defrost(this: *mut Self) -> ManuallyDrop<Arc<Self>> {
        ManuallyDrop::new(unsafe { Arc::from_raw(this) })
    }

    pub fn next_dup(&self) -> Option<Arc<Cell<T>>> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let ptr = links.next.load(Ordering::Acquire);
                if ptr.is_null() {
                    return None;
                }
                let tmp = Cell::defrost(ptr);
                let res = Arc::clone(&*tmp);

                Some(res)
            }
            Dummy(Last) => None,
        }
    }

    pub fn next(&self) -> Option<Arc<Cell<T>>> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let ptr = links.next.load(Ordering::Acquire);
                if ptr.is_null() {
                    return None;
                }
                let tmp = Cell::defrost(ptr);

                Some(ManuallyDrop::into_inner(tmp))
            }
            Dummy(Last) => None,
        }
    }

    pub fn swap_in_next(
        &self,
        p: Arc<Cell<T>>,
        n: Arc<Cell<T>>,
    ) -> Result<Arc<Cell<T>>> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let p_ptr = Arc::as_ptr(&p) as *mut Cell<T>;
                let n_ptr = Arc::as_ptr(&n) as *mut Cell<T>;

                links
                    .next
                    .compare_exchange(p_ptr, n_ptr, Ordering::AcqRel, Ordering::Acquire)
                    .map_err(|ptr| {
                        anyhow!(
                            "[err compare_exchange] actual {:p}, expected {:p}",
                            ptr, p_ptr
                        )
                    })?;

                drop(p);
                n.conserve();
                Ok(ManuallyDrop::into_inner(Cell::defrost(p_ptr)))
            }
            Dummy(Last) => Err(anyhow!("no next for last variant")),
        }
    }

    pub fn next_cmp(&self, target: &Arc<Cell<T>>) -> bool {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let ptr = links.next.load(Ordering::Acquire);
                let target_ptr = Arc::as_ptr(target);
                ptr as *const Cell<T> == target_ptr
            }
            Dummy(Last) => false,
        }
    }
}
