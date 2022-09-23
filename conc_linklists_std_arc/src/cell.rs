use std::mem::ManuallyDrop;
use std::ptr::{self};
use std::sync::Weak;
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

    pub fn delete_chain(self: Arc<Self>) {
        let mut a = self;
        let mut b = a.next_dup();
        a.drop_links();
        while let Some(_b) = b {
            b = _b.next_dup();
            a = _b;
            a.drop_links();
        }
    }

    pub fn drop_links(&self) {

        use self::Cell::*;
        use self::Dummy::*;

        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let ptr = links.next.swap(ptr::null_mut(), Ordering::AcqRel);
                if !ptr.is_null() {
                    let tmp = Cell::defrost(ptr);
                    ManuallyDrop::into_inner(tmp);
                } 
                let ptr = links.back_link.swap(ptr::null_mut(), Ordering::AcqRel);
                if !ptr.is_null() {
                    let tmp = Cell::defrost(ptr);
                    ManuallyDrop::into_inner(tmp);
                } 
            }
            Dummy(Last) => {},
        }
    }
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

    fn _conserve_weak(this: Weak<Self>) -> *mut Self {
        Weak::into_raw(this) as *mut Self
    }
    fn _defrost_weak(this: *mut Self) -> ManuallyDrop<Weak<Self>> {
        ManuallyDrop::new(unsafe {Weak::from_raw(this)})
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
    pub fn store_backlink(&self, backlink: Arc<Self>) {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. }    => {
                let new = backlink.conserve();
                let prev = links.back_link.swap(new, Ordering::AcqRel);
                if prev.is_null() {
                    return;
                }
                let _dropped = ManuallyDrop::into_inner(Cell::defrost(prev)) ;
            }
            Dummy(Last) |  Dummy(First(..)) | Aux { .. } => {},
        }

    }

    pub fn backlink_dup(&self) -> Option<Arc<Self>> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. }    => {
                let prev = links.back_link.load(Ordering::Acquire);
                if prev.is_null() {
                    return None;
                }

                let tmp = Cell::defrost(prev);
                Some(Arc::clone(&*tmp))
            }
            Dummy(Last) |  Dummy(First(..)) | Aux { .. } => None,
        }

    }


    pub fn swap_in_next(
        &self,
        p: Arc<Cell<T>>,
        n: Option<Arc<Cell<T>>>,
    ) -> Result<Arc<Cell<T>>> {
        use self::Cell::*;
        use self::Dummy::*;
        match self {
            Data { ref links, .. } | Aux { ref links } | Dummy(First(ref links)) => {
                let p_ptr = Arc::as_ptr(&p) as *mut Cell<T>;
                let n_ptr = match n {
                    None => ptr::null_mut(),
                    Some(ref _n) => Arc::as_ptr(_n) as *mut Cell<T>,

                };

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
                match n {
                    None => ptr::null_mut(),
                    Some(_n) => _n.conserve(),
                };
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
