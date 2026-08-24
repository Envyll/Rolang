use crate::parser::ExprInd;
use crate::semantics::ro_type::{Replacements, RoType};
use crate::semantics::{
    AnalyzeErr, AnalyzerContext, FnInstanceDescr, FnInstanceMeta, FunctionDefinition,
};
use crate::{ImStr, Multimap};
use std::collections::{HashMap, VecDeque};

const MAX_DEPTH: usize = 1024;

impl AnalyzerContext {
    pub(super) fn monomorphize(&mut self) -> HashMap<FnInstanceDescr, FnInstanceMeta> {
        let mut queue: VecDeque<_> = self.functions.iter().filter_map(take_independent).collect();
        let mut result: HashMap<FnInstanceDescr, FnInstanceMeta> = HashMap::new();

        while let Some(Entry {
            n,
            instance,
            returns,
            next,
            def,
            place,
        }) = queue.pop_front()
        {
            let mut replaces = Replacements::new();
            def.type_on_exit(&instance.args, &returns, &mut replaces);

            if result.contains_key(&instance) {
                continue;
            }

            if n >= MAX_DEPTH {
                self.errors
                    .insert_value(*place, AnalyzeErr::TypeRecursionTooDeep);
                continue;
            }

            next.into_iter().for_each(|(descr, ty, inds)| {
                let def = &self.functions[&descr.name];
                queue.push_back(Entry {
                    n: n.strict_add(1),
                    instance: descr.clone(),
                    returns: ty.clone(),
                    def,
                    next: def
                        .runs
                        .iter()
                        .filter_map(|(next_descr_gen, next_ret_gen, next_expr)| {
                            let ref next_def = self.functions[&next_descr_gen.name];
                            let next_args = next_def
                                .args
                                .iter()
                                .map(|ty| ty.deep_clone(&mut replaces))
                                .collect();

                            let next_descr = FnInstanceDescr {
                                name: next_descr_gen.name.clone(),
                                args: next_args,
                            };

                            if result.contains_key(&next_descr) {
                                None?
                            }

                            let next_ret = next_ret_gen.deep_clone(&mut replaces);

                            Some((next_descr, next_ret, next_expr))
                        })
                        .collect(),
                    place: inds,
                })
            });

            if let Some(l) = result.get(&instance) {
                let ok = l.ty.try_make_like(&returns);
                if !ok {
                    //self.errors[place.0] = to/do
                }
            } else {
                result.insert(
                    instance,
                    FnInstanceMeta {
                        ty: returns,
                        replacements: replaces.clone(),
                    },
                );
            };
        }

        result
    }
}

struct Entry<'a> {
    n: usize,
    instance: FnInstanceDescr,
    returns: RoType,
    next: Vec<(FnInstanceDescr, RoType, &'a ExprInd)>,
    def: &'a FunctionDefinition,
    place: &'a ExprInd,
}

fn take_independent<'a>((name, f): (&ImStr, &'a FunctionDefinition)) -> Option<Entry<'a>> {
    let mut repl = Replacements::default();

    let args = f.args.iter().map(|ty| ty.deep_clone(&mut repl)).collect();

    //dbg!(&name, &args, &f.args);

    if repl
        .values()
        .filter_map(|v| v.try_unwrap_maybe_unknown_ref().ok())
        .any(|rc| rc.0.get().is_none())
    {
        None?
    }

    Some(Entry {
        n: 0,
        returns: f.return_type.clone(),
        next: f
            .runs
            .iter()
            .map(|(descr, ty, place)| (descr.clone(), ty.clone(), place))
            .collect(),
        def: f,
        instance: FnInstanceDescr {
            name: name.clone(),
            args,
        },
        place: &f.expr,
    })
}
