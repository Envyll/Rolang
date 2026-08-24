use crate::semantics::analyzer::analyze::Unit;
use crate::semantics::analyzer::{AnalyzerContext, AndThen, CurrCtx};
use crate::semantics::ro_type::RoType;
use crate::ImStr;
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;
use std::rc::Rc;

impl AnalyzerContext {
    #[inline]
    pub(super) fn defer<'a>(&mut self, f: impl FnOnce(&mut Self) -> Unit + 'static) -> Unit {
        self.tasks.push_back(Box::new(f));
        Unit
    }

    #[inline]
    pub(super) fn defer_until_fn_done(
        &mut self,
        name: ImStr,
        f: impl FnOnce(&mut Self) -> Unit + 'static,
    ) -> Unit {
        match self.fn_tasks.get_mut(&name) {
            None => self.defer(f),
            Some(fs) => {
                fs.push(Box::new(f));
                Unit
            }
        }
    }

    #[inline]
    pub(super) fn defer_after_all(
        &mut self,
        ctx: CurrCtx,
        tasks: impl IntoIterator<
            IntoIter = impl Iterator<Item = impl FnOnce(&mut Self, CurrCtx, AndThen) -> Unit + 'static>
                       + ExactSizeIterator,
        >,
        f: impl FnOnce(&mut Self, Box<[RoType]>, CurrCtx) -> Unit + 'static,
    ) -> Unit {
        let tasks = tasks.into_iter();
        if tasks.len() == 0 {
            return self.defer(move |this| f(this, Box::new([]), ctx));
        }

        let f_shared = Rc::new((
            Cell::new(tasks.len()),
            Cell::new(Some(f)),
            // cell of vec of tasks.len uninits of rotype
            RefCell::new(
                (0..tasks.len())
                    .map(|_| MaybeUninit::uninit())
                    .collect::<Box<[_]>>(),
            ),
        ));

        for (i, task) in tasks.enumerate() {
            let ctx = ctx.clone();
            let f_shared = f_shared.clone();
            let Unit = self.defer(move |this| {
                task(
                    this,
                    ctx,
                    Box::new(move |this: &mut Self, ty, ctx| {
                        let tasks_left = f_shared.0.replace(f_shared.0.get() - 1);
                        f_shared.2.borrow_mut()[i].write(ty);
                        // future : use some combination of contexts instead

                        // is it the last task?
                        if tasks_left == 1 {
                            let res = unsafe { f_shared.2.take().assume_init() };
                            let f = f_shared.1.take().unwrap();

                            this.defer(move |this| f(this, res, ctx))
                        } else {
                            Unit
                        }
                    }),
                )
            });
        }
        Unit
    }

    #[inline]
    pub(super) fn defer_after_one(
        &mut self,
        result_ty: RoType,
        ctx: CurrCtx,
        tasks: impl IntoIterator<
            IntoIter = impl Iterator<
                Item = impl FnOnce(&mut Self, CurrCtx, /*result*/ RoType, AndThen) -> Unit + 'static,
            > + ExactSizeIterator,
        >,
        f: impl FnOnce(&mut Self, RoType, CurrCtx) -> Unit + 'static,
    ) -> Unit {
        let tasks = tasks.into_iter();
        let len = tasks.len();
        let f_shared = Rc::new((
            // closure
            RefCell::new(Some(f)),
            // branches incomplete
            Cell::new(len + 1),
        ));

        let and_then_for_last = move |this: &mut Self, ty, ctx| {
            f_shared.1.update(|i| i - 1);

            if f_shared.1.get() == 0
                && let Some(f) = f_shared.0.take()
            {
                // future: use not just the last context
                f(this, ty, ctx)
            } else {
                Unit
            }
        };

        tasks.for_each(|task| {
            // future : maybe it will not be so exact…
            let ctx = ctx.clone();
            let result_ty = result_ty.clone();
            let and_then_for_last = Box::new(and_then_for_last.clone());

            let _ = self.defer(move |this| task(this, ctx, result_ty, and_then_for_last));
        });

        // for the case if `tasks.len() == 0`
        and_then_for_last(self, result_ty.clone(), ctx.clone())
    }
}
