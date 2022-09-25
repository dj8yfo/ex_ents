use anyhow::{Result, anyhow, Context};

use crate::cell::Cell;

use super::Cursor;
use std::{fmt::Debug, sync::Arc};

type _3Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>, Arc<Cell<T>>);
type _2Cells<T> = (Arc<Cell<T>>, Arc<Cell<T>>);
impl<T: Debug> Cursor<T> {

    #[allow(dead_code)]
    fn outlink_target(&mut self) -> Result<_2Cells<T>> {
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


        self.target.take();
        Ok((d,  n))
    }

    #[allow(dead_code)]
    fn calculate_delete_start(&self) -> Result<_2Cells<T>> {

        let mut p = self.pre_cell.clone();
        while let Some(q) = p.backlink_dup() {
            p = q;
        }
        let s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok((p, s))
    }

    #[allow(dead_code)]
    fn n_is_last_aux(n: &Arc<Cell<T>>) -> Result<bool> {
        let n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        Ok(n_next.is_normal_cell())
    }

    #[allow(dead_code)]
    fn advance_delete_end(mut n: Arc<Cell<T>>) -> Result<Arc<Cell<T>>> {
        let mut n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        while !n_next.is_normal_cell() {
            n = n_next;
            n_next = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        }
        Ok(n)
    }

    #[allow(dead_code)]
    pub fn try_delete(&mut self) -> Result<()> {
        let (target_dropped, mut n) = self.outlink_target()?;

        let (p, mut s) = self.calculate_delete_start()?;
        target_dropped.store_backlink(Some(Arc::downgrade(&p) ));


        n = Cursor::advance_delete_end(n)?;
        loop {
            let res = p.swap_in_next(s.clone(), Some(n.clone()));
            if res.is_err(){
                s = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
            }

            match DeleteLoopCondition::new(res.is_ok(), &p, &n)? {
                Failure => {},
                Success | ConcurrentDelForward | ConcurrentDelPrev => break,
            }
        }

        Ok(())
    }
}

use DeleteLoopCondition::*;

enum DeleteLoopCondition {
    Success, 
    Failure,
    ConcurrentDelForward,
    ConcurrentDelPrev,
}

impl DeleteLoopCondition {
    fn new<T:Debug>(res: bool, p: &Arc<Cell<T>>, n: &Arc<Cell<T>>) -> Result<Self> {
        if res {
            return Ok(Self::Success);
        }
        if p.backlink_dup().is_some() {
            return Ok(Self::ConcurrentDelPrev);
        }
        if !Cursor::n_is_last_aux(n)? {
            return Ok(Self::ConcurrentDelForward);

        }

        Ok(Self::Failure)

    } 
}

