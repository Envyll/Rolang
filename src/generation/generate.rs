use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::parser::ExprInd;
use crate::semantics::patterns::{PatternColumnData, PatternColumnOrder, PatternTable};
use crate::semantics::types::RoType;
use crate::semantics::{AnalyzedAST, FunctionDefinition, VariableDefinition, };
use crate::AResult;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue};
use itertools::Itertools;
use std::collections::HashMap;

type Vars<'a> = HashMap<VariableDefinition, BasicValueEnum<'a>>;
type Fns<'a> = HashMap<FunctionDefinition, FunctionValue<'a>>;

impl AnalyzedAST {
    pub fn build_module(&self, module: &Module, ) -> AResult {
        let mut fns = HashMap::new();

        for (name, def) in &self.functions {
            let out_ty = def
                .return_type
                .get()
                .unwrap()
                .basic_type(module.get_context());
            let ty = module.get_context().i64_type();
            let f_ty = BasicType::fn_type(&out_ty, &vec![ty.into(); def.args.len()], false);

            let f = module.add_function(name.as_str(), f_ty, Some(Linkage::External));
            //f.set_call_conventions(18);

            fns.insert(def.clone(), f.into());
        }

        let ref mut ctx = BuildContext {
            module,
            builder: module.get_context().create_builder(),
            vars: Default::default(),
            fns,
        };

        for def in self.functions.values() {
            self.build_fn(ctx, def)?;
        }

        //module.print_to_stderr();
        module.verify().map_err(|err| anyhow::anyhow!("{err}"))?;


        Ok(())
    }

    fn build_fn(&self, ctx: &mut BuildContext, def: &FunctionDefinition) -> AResult {
        let f = ctx.fns[&def];

        ctx.builder
            .position_at_end(ctx.module.get_context().append_basic_block(f, ""));

        let (_, _, right) = self.ast.at(def.expr).is_un_flow().unwrap();

        let mut value = None;

        for expr in right {
            for (par_name, def) in &def.args {
                let &VariableDefinition::Variable { n, .. } = def else {
                    unimplemented!()
                };
                let param = f.get_nth_param(n).unwrap();
                param.set_name(par_name.as_str());
                ctx.vars.insert(def.clone(), param.into());
            }
            value = self.build_expr(ctx, expr)?.into();
        }
        ctx.builder.build_return(Some(&value.unwrap()))?;
        Ok(())
    }

    fn build_expr<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        expr: ExprRef,
    ) -> AResult<BasicValueEnum<'a>> {
        Ok(match expr.ty() {
            ExprRefType::Empty => {
                unimplemented!()
            }
            ExprRefType::Simple(_) => match &self.references[expr.ind().0] {
                VariableDefinition::Number(n) => ctx
                    .module
                    .get_context()
                    .i64_type()
                    .const_int(*n, false)
                    .into(),
                def @ VariableDefinition::Variable { .. } => ctx.vars[def],
                _ => unimplemented!(),
            },
            ExprRefType::Unary(op, r) => match op.slice.as_str() {
                "+" | "*" => self.build_expr(ctx, r)?.into_int_value().into(),
                "-" => {
                    let r_value = self.build_expr(ctx, r)?.into_int_value();
                    ctx.builder.build_int_neg(r_value, "")?.into()
                }
                _ => unimplemented!(),
            },
            ExprRefType::Binary(l, op, r) => {
                let l_value = self.build_expr(ctx, l)?.into_int_value();
                let r_value = self.build_expr(ctx, r)?.into_int_value();
                match op.slice.as_str() {
                    "+" => ctx.builder.build_int_add(l_value, r_value, "")?.into(),
                    "*" => ctx.builder.build_int_mul(l_value, r_value, "")?.into(),
                    "-" => ctx.builder.build_int_sub(l_value, r_value, "")?.into(),
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Path(_, _) => {
                unimplemented!()
            }
            ExprRefType::Apply(mut exprs) => {
                let ExprRefType::Simple(f) = exprs.next().unwrap().ty() else {
                    unimplemented!()
                };

                let mut expr_values: Vec<BasicMetadataValueEnum> = vec![];

                for expr in exprs {
                    expr_values.push(self.build_expr(ctx, expr)?.into());
                }

                match &self.references[expr.ind().0] {
                    &VariableDefinition::Member(n) => ctx.builder.build_extract_value(
                        expr_values.first().unwrap().into_struct_value(),
                        n,
                        "",
                    )?,
                    VariableDefinition::Function { .. } => {
                        let call: CallSiteValue = ctx.builder.build_call(
                            ctx.fns[&self.functions[f.slice_or_name()]],
                            &expr_values,
                            "",
                        )?;
                        call.set_tail_call(true);
                        call.try_as_basic_value().basic().unwrap()
                    }
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Braced(_, exprs, _) => {
                let mut value = None;

                for expr in exprs {
                    value = Some(self.build_expr(ctx, expr)?);
                }

                value.unwrap_or_else(|| {
                    ctx.module
                        .get_context()
                        .struct_type(&[], false)
                        .get_undef()
                        .into()
                })
            }
            ExprRefType::Flow(l, _t, _m, _r) => {
                let val = self.build_expr(ctx, l)?;
                if let VariableDefinition::Match { branches, patterns, ty, pred_ty } = &self.references[expr.ind().0] {
                    self.build_match(ctx, val, &branches, &patterns, ty, pred_ty)?
                } else {
                    unimplemented!();
                }
            }
            ExprRefType::UnFlow(_, l, r) => {
                let ExprRefType::Simple(l) = l.unwrap().ty() else {
                    unimplemented!()
                };

                let mut value = None;

                for expr in r {
                    value = Some(self.build_expr(ctx, expr)?);
                }
                let value = value.unwrap();
                if value.get_name().is_empty() {
                    value.set_name(l.slice_or_name());
                }

                ctx.vars.insert(
                    VariableDefinition::Variable {
                        place: expr.ind(),
                        name: l.slice_or_name().clone(),
                        n: 0,
                        ty: RoType::Int,
                    },
                    value.into(),
                );
                value
            }
            ExprRefType::Tuple(m) => {
                let ty = self.references[expr.ind().0]
                    .ty()
                    .basic_type(ctx.module.get_context())
                    .into_struct_type();

                let mut val = ty.get_undef();

                for (i, expr) in m.enumerate() {
                    let value = self.build_expr(ctx, expr)?;
                    val = ctx
                        .builder
                        .build_insert_value(val, value, i as u32, "")?
                        .into_struct_value();
                }

                val.into()
            }
            ExprRefType::Either(_) => {
                unimplemented!()
            }
        })
    }

    fn build_match<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        branches: &[ExprInd],
        patterns: &PatternTable,
        ret_ty: &RoType,
        pred_ty: &RoType,
    ) -> AResult<BasicValueEnum<'a>> {
        //dbg!(patterns);

        let curr_block = ctx.builder.get_insert_block().unwrap();

        let unreachable_block = ctx.module.get_context().insert_basic_block_after(curr_block, "match.unreachable");
        ctx.builder.position_at_end(unreachable_block);
        ctx.builder.build_unreachable()?;

        let end_block = ctx
            .module
            .get_context()
            .insert_basic_block_after(curr_block, "match.end");


        ctx.builder.position_at_end(end_block);
        let phi = ctx
            .builder
            .build_phi(ret_ty.basic_type(ctx.module.get_context()), "")?;

        let mut blocks = Vec::with_capacity(branches.len());

        for branch in branches {
            let (_tok, _l, r) = self.ast.at(*branch).is_un_flow().unwrap();

            let block = ctx
                .module
                .get_context()
                .insert_basic_block_after(curr_block, "match");
            ctx.builder.position_at_end(block);

            let mut value = ctx
                .module
                .get_context()
                .struct_type(&[], false)
                .get_undef()
                .into();

            for expr in r {
                value = self.build_expr(ctx, expr)?;
            }

            ctx.builder.build_unconditional_branch(end_block)?;

            phi.add_incoming(&[(&value, block)]);

            blocks.push(block);
        }

        let ref mut empty_blocks = vec![];

        ctx.builder.position_at_end(curr_block);
        self.build_match_branch(ctx, val, &blocks, patterns.elements.iter(), &patterns.branches, pred_ty, &(0..patterns.branches.len()).collect_vec(), unreachable_block, empty_blocks)?;

        for (from, to) in empty_blocks {
            from.replace_all_uses_with(to);
            unsafe { from.delete().unwrap(); }
        }

        ctx.builder.position_at_end(end_block);
        Ok(phi.as_basic_value())
    }

    fn build_match_branch<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        blocks: &[BasicBlock<'a>],
        mut patterns: std::collections::btree_map::Iter<PatternColumnOrder, PatternColumnData>,
        branches: &[usize],
        ty: &RoType,
        mut selections: &[usize],
        else_br: BasicBlock<'a>,
        empty_blocks: &mut Vec<(BasicBlock<'a>, BasicBlock<'a>)>,
    ) -> AResult {
        let patterns2 = patterns.clone();

        macro_rules! redirect {
            ($expr:expr) => {
                let expr = $expr;
                let block = ctx.builder.get_insert_block().unwrap();

                if block.get_first_instruction().is_none() {
                    empty_blocks.push((block, expr));
                }

                ctx.builder
                    .build_unconditional_branch(expr)?;
            };
        }

        match patterns.next() {
            None => {
                if !selections.is_empty() {
                    redirect!(blocks[branches[selections[0]]]);
                    return Ok(());
                }
            }
            Some((ord, PatternColumnData::Int { values, ignore })) => {
                let switch_val = ord.take_value(val, &ctx.builder, ty)?.into_int_value();
                while !selections.is_empty() {
                    let curr_br = ctx.builder.get_insert_block().unwrap();
                    let fallback_br = ctx
                        .module
                        .get_context()
                        .insert_basic_block_after(curr_br, "else");

                    let mut i = 0;
                    while i != selections.len() && ignore[selections[i]] {
                        i += 1;
                    }

                    let mut map = HashMap::<u64, (Vec<_>, BasicBlock)>::default();

                    let mut j = i;
                    while j != selections.len() && !ignore[selections[j]] {
                        if let Some((v, _)) = map.get_mut(&values[selections[j]]) {
                            v.push(selections[j])
                        } else {
                            map.insert(
                                values[selections[j]],
                                (
                                    vec![selections[j]],
                                    ctx.module
                                        .get_context()
                                        .insert_basic_block_after(curr_br, "value"),
                                ),
                            );
                        }
                        j += 1;
                    }

                    let blanks = &selections[0..i];
                    let left = &selections[j..];

                    let mut cases = vec![];

                    for (value, (ns, block)) in map {
                        ctx.builder.position_at_end(block);
                        self.build_match_branch(
                            ctx,
                            val,
                            blocks,
                            patterns.clone(),
                            branches,
                            ty,
                            &ns,
                            fallback_br,
                            empty_blocks,
                        )?;
                        cases.push((switch_val.get_type().const_int(value, false), block));
                    }

                    let then_br = ctx
                        .module
                        .get_context()
                        .insert_basic_block_after(curr_br, "match.switch");
                    ctx.builder.position_at_end(then_br);

                    ctx.builder.build_switch(switch_val, fallback_br, &cases)?;

                    ctx.builder.position_at_end(curr_br);
                    self.build_match_branch(
                        ctx,
                        val,
                        blocks,
                        patterns.clone(),
                        branches,
                        ty,
                        blanks,
                        then_br,
                        empty_blocks,
                    )?;

                    ctx.builder.position_at_end(fallback_br);
                    patterns = patterns2.clone();
                    selections = left;
                }
            }
        }

        redirect!(else_br);
        Ok(())
    }

}

#[derive(Debug)]
struct BuildContext<'a: 'b, 'b> {
    module: &'b Module<'a>,
    builder: Builder<'a>,
    vars: Vars<'a>,
    fns: Fns<'a>,
}
