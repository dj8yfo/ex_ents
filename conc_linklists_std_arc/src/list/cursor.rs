use std::{fmt::Debug, sync::Arc};

use crate::cell::Cell;
use anyhow::{Result, anyhow, Context};

pub struct Cursor<T: Debug> {
    pub(super) target: Option<Arc<Cell<T>>>,
    pub(super) pre_aux: Arc<Cell<T>>,
    pub(super) pre_cell: Arc<Cell<T>>,
}

impl<T: Debug> Cursor<T> {
    pub fn new(pre_cell: Arc<Cell<T>>, pre_aux: Arc<Cell<T>>) -> Self {
        Self {
            target: None,
            pre_cell,
            pre_aux,
        }
    }

    pub fn update(&mut self) {
        match self.target {
            None => {}
            Some(ref target) => {
                if self.pre_aux.next_cmp(target) {
                    return;
                }
            }
        }

        let mut p = self.pre_aux.clone(); // expecting aux variant
        let mut n = p.next_dup().unwrap();

        drop(self.target.take());
        while !n.is_last() && !n.is_data_cell() {

            if let Err(msg) = self.pre_cell.swap_in_next(p, n.clone()) {
                debug_assert!({
                    println!("cursor.update {}", msg);
                    true
                })
            }

            p = n.clone();
            n = n.next_dup().unwrap();
        }
        self.pre_aux = p;
        self.target = Some(n);
    }

    pub fn try_insert(&self, data: T) -> Result<()> {
        if self.target.is_none() {
            return Err(anyhow!("target is none; cursor needs updating"));
        }
        let target_ref = self.target.as_ref().unwrap();
        let aux = Cell::new_aux(target_ref.clone()); // +1 target
        let err_ctx = format!("err on try_insert {:?}", data);
        let data = Cell::new_data(data, aux);

        let res = self.pre_aux.swap_in_next(target_ref.clone(), data).with_context(||
            {
                err_ctx
            })?;
        drop(res); // -1 target
        Ok(())
    }
}
