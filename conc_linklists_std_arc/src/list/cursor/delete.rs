use anyhow::{Result, anyhow, Context};

use crate::cell::Cell;

use super::Cursor;
use std::{fmt::Debug, sync::Arc};

type _3Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>, Arc<Cell<T>>);
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

    fn calculate_delete_start(&self) -> Result<_2Cells<T>> {

        let mut p = self.pre_cell.clone();
        while let Some(q) = p.backlink_dup() {
            p = q;
        }
        let s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok((p, s))
    }

    fn n_is_last_aux(n: Arc<Cell<T>>) -> Result<bool> {
        let mut n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok(n_next.is_normal_cell())
    }

    fn advance_delete_end(mut n: Arc<Cell<T>>) -> Result<Arc<Cell<T>>> {
        let mut n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        while !n_next.is_normal_cell() {
            n = n_next;
            n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        }
        Ok(n)
    }
    //
    // fn compute_loop_result(res: bool, ) -> DeleteLoopState {
    //     if res {
    //
    //     }
    //
    // }
    pub fn try_delete(&mut self) -> Result<()> {
        let (target_dropped, mut n) = self.drop_target()?;

        let (p, mut s) = self.calculate_delete_start()?;


        n = Cursor::advance_delete_end(n)?;
        let loop_res = loop {
            let res = p.swap_in_next(s.clone(), Some(n.clone()));
            if res.is_err(){
                s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
            }
            break;
        };


        // delete backchain
        // HACK: p_back_prev.store_backlink(None)
        // HACK: target_dropped.delete_chain_back()


        Ok(())
    }
}

