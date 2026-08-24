use crate::AResult;
use crate::generation::generate::BuildContext;
use crate::parser::ExprInd;
use crate::semantics::patterns::{Bindings, PatternColumnData, PatternColumnOrder, PatternTable};
use crate::semantics::ro_type::RoType;
use crate::semantics::{AnalyzedAST, BindingReference};
use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, PhiValue};
use intmap::IntMap;
use itertools::Itertools;
use rpds::Stack;
use std::collections::HashMap;
use std::ops::Deref;

impl AnalyzedAST {
    pub(super) fn build_let<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        patterns: &PatternTable,
        pred_ty: &RoType,
        name: &str,
    ) -> AResult {
        let curr_block = ctx.builder.get_insert_block().unwrap();

        let end_block = ctx
            .module
            .get_context()
            .insert_basic_block_after(curr_block, &format!("{name}.in"));

        ctx.builder.position_at_end(curr_block);

        self.build_match_branch(
            ctx,
            val,
            &[end_block],
            patterns,
            pred_ty,
            ctx.unreachable,
            name,
        )?;

        ctx.builder.position_at_end(end_block);

        Ok(())
    }

    pub(super) fn build_match<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        branches: &[ExprInd],
        patterns: &PatternTable,
        ret_ty: &RoType,
        pred_ty: &RoType,
        name: &str,
    ) -> AResult<BasicValueEnum<'a>> {
        //dbg!(patterns);

        let curr_block = ctx.builder.get_insert_block().unwrap();

        let end_block = ctx
            .module
            .get_context()
            .insert_basic_block_after(curr_block, &format!("{name}.end"));

        ctx.builder.position_at_end(end_block);
        let phi = ctx.builder.build_phi(
            ret_ty.basic_type(ctx.module.get_context()),
            &format!("{name}.result"),
        )?;

        let blocks = (0..branches.len())
            .map(|_| {
                ctx.module
                    .get_context()
                    .insert_basic_block_after(curr_block, &format!("{name}.block"))
            })
            .collect_vec();

        ctx.builder.position_at_end(curr_block);
        self.build_match_branch(ctx, val, &blocks, patterns, pred_ty, ctx.unreachable, name)?;

        for (&branch, block) in branches.iter().zip_eq(blocks) {
            let (.., r) = self.ast.at(branch).is_un_flow().unwrap();

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
        }

        ctx.builder.position_at_end(end_block);
        Ok(phi.as_basic_value())
    }

    pub(super) fn build_match_branch<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        branches_blocks: &[BasicBlock<'a>],
        patterns: &PatternTable,
        pred_ty: &RoType,
        else_br: BasicBlock<'a>,
        name: &str,
    ) -> AResult {
        self.build_match_branch_internal(
            ctx,
            val,
            branches_blocks,
            &mut Default::default(),
            Stack::from_iter(patterns.elements.iter()),
            &patterns.branches,
            &patterns.bindings,
            pred_ty,
            &(0..patterns.branches.len()).collect_vec(),
            else_br,
            name,
        )?;

        Ok(())
    }

    fn build_match_branch_internal<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        val: BasicValueEnum<'a>,
        blocks: &[BasicBlock<'a>],
        phi: &mut HashMap<BindingReference, PhiValue<'a>>,
        patterns: Stack<(&PatternColumnOrder, &PatternColumnData)>,
        branches: &[usize],
        bindings: &<Bindings as Deref>::Target,
        ty: &RoType,
        mut selections: &[usize], // sub-branches numbers we are working with
        else_br: BasicBlock<'a>,
        name: &str,
    ) -> AResult {
        macro_rules! redirect {
            ($expr:expr) => {
                ctx.builder.build_unconditional_branch($expr)?;
            };
        }

        match patterns.peek() {
            // if nothing left, jump to the match branch block
            None if let Some(&selection) = selections.first() => {
                let block = blocks[branches[selection]];

                let curr_block = ctx.builder.get_insert_block().unwrap();

                for (_, binding) in bindings[selection].iter() {
                    let pos = self.references[binding.expr.0].unwrap_variable_ref();

                    let val = binding
                        .pattern_column_position
                        .take_value(val, &ctx.builder, ty)?;

                    let phi = if let Some(phi) = phi.get(pos) {
                        *phi
                    } else {
                        ctx.builder.position_at_end(block);
                        let phi_new = ctx.builder.build_phi(val.get_type(), &pos.name)?;
                        ctx.builder.position_at_end(curr_block);

                        phi.insert(pos.clone(), phi_new);
                        ctx.vars.insert(pos.clone(), phi_new.as_basic_value());
                        phi_new
                    };
                    phi.add_incoming(&[(&val, curr_block)]);
                }

                redirect!(block);
                return Ok(());
            }
            None => {}
            Some((ord, PatternColumnData::Int { values, ignore })) => {
                let switch_val = ord.take_value(val, &ctx.builder, ty)?.into_int_value();
                while !selections.is_empty() {
                    let curr_br = ctx.builder.get_insert_block().unwrap();
                    let fallback_br = ctx
                        .module
                        .get_context()
                        .insert_basic_block_after(curr_br, &format!("{name}.else"));

                    let ignore_cnt = selections.iter().take_while(|&&i| ignore[i]).count();

                    let mut map = IntMap::<u64, (Vec<_>, BasicBlock)>::default();

                    let mut not_ignore_ind = ignore_cnt;
                    while not_ignore_ind != selections.len() && !ignore[selections[not_ignore_ind]]
                    {
                        if let Some((v, _)) = map.get_mut(values[selections[not_ignore_ind]]) {
                            v.push(selections[not_ignore_ind])
                        } else {
                            map.insert(
                                values[selections[not_ignore_ind]],
                                (
                                    vec![selections[not_ignore_ind]],
                                    ctx.module.get_context().insert_basic_block_after(
                                        curr_br,
                                        &format!("{name}.value"),
                                    ),
                                ),
                            );
                        }
                        not_ignore_ind += 1;
                    }

                    let blanks = &selections[..ignore_cnt];
                    let left = &selections[not_ignore_ind..];

                    let mut cases = vec![];

                    for (value, (ns, block)) in map {
                        ctx.builder.position_at_end(block);
                        self.build_match_branch_internal(
                            ctx,
                            val,
                            blocks,
                            phi,
                            patterns.pop().unwrap(),
                            branches,
                            bindings,
                            ty,
                            &ns,
                            fallback_br,
                            name,
                        )?;
                        cases.push((switch_val.get_type().const_int(value, false), block));
                    }

                    let then_br = ctx
                        .module
                        .get_context()
                        .insert_basic_block_after(curr_br, &format!("{name}.switch"));
                    ctx.builder.position_at_end(then_br);

                    ctx.builder.build_switch(switch_val, fallback_br, &cases)?;

                    ctx.builder.position_at_end(curr_br);
                    self.build_match_branch_internal(
                        ctx,
                        val,
                        blocks,
                        phi,
                        patterns.pop().unwrap(),
                        branches,
                        bindings,
                        ty,
                        blanks,
                        then_br,
                        name,
                    )?;

                    ctx.builder.position_at_end(fallback_br);
                    selections = left;
                }
            }
        }

        redirect!(else_br);
        Ok(())
    }
}
