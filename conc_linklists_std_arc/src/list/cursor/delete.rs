use anyhow::{Result, anyhow, Context};

use crate::cell::Cell;

use super::Cursor;
use std::{fmt::Debug, sync::Arc};

type Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>, Arc<Cell<T>>);
impl<T: Debug> Cursor<T> {

    fn drop_target(&mut self) -> Result<Cells<T>> {
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

        Ok((self.pre_aux.clone(), d,  n))
    }

    pub fn try_delete(&mut self) -> Result<()> {
        let (mut n_prev, target_dropped, mut n) = self.drop_target()?;


        let mut p_back_prev = target_dropped.clone();
        // HACK: p_back_prev.store_backlink(None)
        // HACK: target_dropped.delete_chain_back()
        let mut p = self.pre_cell.clone();
        while let Some(q) = p.backlink_dup() {
            p_back_prev = p;
            p = q;
        }
        let s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        Ok(())
    }
}
