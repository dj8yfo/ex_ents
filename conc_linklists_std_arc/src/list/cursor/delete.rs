use anyhow::{Result, anyhow, Context};

use crate::cell::Cell;

use super::Cursor;
use std::{fmt::Debug, sync::Arc};

type _4Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>, Arc<Cell<T>>, Arc<Cell<T>>);
type _2Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>);
impl<T: Debug> Cursor<T> {

    fn drop_target(&mut self) -> Result<_2Cells<T>> {
        let target = match self.target {
            None => return Err(anyhow!("target is none; cursor needs updating")),
            Some(ref _target) => _target,
        };

        if target.is_last() {
            return Err(anyhow!("target is last; no possibility to delete"));
        }

        let d = target.clone();
        let n = target.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        self
            .pre_aux
            .swap_in_next(d.clone(), Some(n.clone()))
            .with_context(|| "err on try_delete:drop_target ; cursor needs update")?;

        target.store_backlink(Some(self.pre_cell.clone()));
        self.target.take();

        Ok((d,  n))
    }

    fn calculate_delete_start(&self, target_dropped: Arc<Cell<T>>) -> Result<_4Cells<T>> {

        let mut p_back_prev = target_dropped;
        let mut p = self.pre_cell.clone();
        while let Some(q) = p.backlink_dup() {
            p_back_prev = p;
            p = q;
        }
        let s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        let s_next = s.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok((p_back_prev, p, s, s_next))
    }

    fn n_is_last_aux(n: Arc<Cell<T>>) -> Result<bool> {
        let mut n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok(n_next.is_normal_cell())
    }

    fn advance_delete_end(mut n: Arc<Cell<T>>) -> Result<_2Cells<T>> {
        let mut n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        while !n_next.is_normal_cell() {
            n = n_next;
            n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        }
        Ok((n, n_next))
    }

    pub fn try_delete(&mut self) -> Result<()> {
        let (target_dropped, mut n) = self.drop_target()?;

        let (p_back_prev, p, s, s_next) = self.calculate_delete_start(target_dropped.clone())?;


        n, n_next = Cursor::advance_delete_end(n)?;
        let loop_res = loop {
            let res = s.swap_in_next(s_next.clone(), Some(n_next.clone()));
            if res.is_err(){
                s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
            }
        };


        // delete backchain
        // HACK: p_back_prev.store_backlink(None)
        // HACK: target_dropped.delete_chain_back()


        Ok(())
    }
}

struct DeleteState {
    result : bool,
    n_is_last_aux: bool,

    

}

