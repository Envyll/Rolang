use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::parser::ExprInd;
use crate::semantics::types::RoType;
use crate::semantics::{patterns::Pattern, AnalyzedAST, FunctionDefinition, VariableDefinition};
use inkwell::builder::{Builder, BuilderError};
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue};
use std::collections::HashMap;

type Vars<'a> = HashMap<VariableDefinition, BasicValueEnum<'a>>;
type Fns<'a> = HashMap<FunctionDefinition, FunctionValue<'a>>;

impl AnalyzedAST {
    pub fn build_module(&self, module: &Module) -> Result<(), BuilderError> {

        let mut fns = HashMap::new();
        
        
        for (name, def) in &self.functions {
            let out_ty = def.return_type.get().unwrap().basic_type(module.get_context());
            let ty = module.get_context().i64_type();
            let f_ty = BasicType::fn_type(&out_ty, &vec![ty.into(); def.args.len()], false);

            let f = module.add_function(name.as_str(), f_ty, Some(Linkage::External));
            //f.set_call_conventions(18);
            
            fns.insert(def.clone(), f.into());
        }
        for def in self.functions.values() {
            self.build_fn(module, def, &mut fns)?;
        }
        
        module.verify().unwrap();

        Ok(())
    }

    fn build_fn<'a>(&self, module: &Module<'a>, def: &FunctionDefinition, fns: &mut Fns<'a>) -> Result<(), BuilderError> {
        let mut vars = Vars::<'a>::new();
        let f = fns[&def];
        let builder = module.get_context().create_builder();
        
        builder.position_at_end(module.get_context().append_basic_block(f, ""));

        let (_, _, right) = self.ast.at(def.expr).is_un_flow().unwrap();

        let mut value = None;

        for expr in right {
            for (par_name, def) in &def.args {
                let &VariableDefinition::Variable {n, ..} = def else {unimplemented!()};
                let param = f.get_nth_param(n).unwrap();
                param.set_name(par_name.as_str());
                vars.insert(def.clone(), param.into());
            };
            value = self.build_expr(BuildContext{ module, function: f, builder: & builder}, expr, &mut vars, fns)?.into();
        }
        builder.build_return(Some(&value.unwrap()))?;
        Ok(())
    }

    fn build_expr<'a>(&self, ctx : BuildContext<'a, '_>, expr: ExprRef, vars: &mut Vars<'a>, fns: &mut Fns<'a>) -> Result<BasicValueEnum<'a>, BuilderError> {
        Ok(match expr.ty() {
            ExprRefType::Empty => { unimplemented!() }
            ExprRefType::Simple(_) => {
                match &self.references[expr.ind().0] {
                    VariableDefinition::Number(n) => ctx.module.get_context().i64_type().const_int(*n, false).into(),
                    def @ VariableDefinition::Variable { .. } => vars[def],
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Unary(op, r) => {
                match op.slice.as_str() {
                    "+" | "*" => {
                        self.build_expr(ctx, r, vars, fns)?.into_int_value().into()
                    }
                    "-" => {
                        let r_value = self.build_expr(ctx, r, vars, fns)?.into_int_value();
                        ctx.builder.build_int_neg(r_value, "")?.into()
                    }
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Binary(l, op, r) => {
                let l_value = self.build_expr(ctx, l, vars, fns)?.into_int_value();
                let r_value = self.build_expr(ctx, r, vars, fns)?.into_int_value();
                match op.slice.as_str() {
                    "+" => {
                        ctx.builder.build_int_add(l_value, r_value, "")?.into()
                    }
                    "*" => {
                        ctx.builder.build_int_mul(l_value, r_value, "")?.into()
                    }
                    "-" => {
                        ctx.builder.build_int_sub(l_value, r_value, "")?.into()
                    }
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Path(_, _) => {unimplemented!()}
            ExprRefType::Apply(mut exprs) => {
                let ExprRefType::Simple(f) = exprs.next().unwrap().ty() else {unimplemented!()};


                let mut expr_values : Vec<BasicMetadataValueEnum> = vec![];

                for expr in exprs {
                    expr_values.push(self.build_expr(ctx, expr, vars, fns)?.into());
                };

                match &self.references[expr.ind().0] {
                    &VariableDefinition::Member(n) => {
                        ctx.builder.build_extract_value(expr_values.first().unwrap().into_struct_value(), n, "")?
                    },
                    VariableDefinition::Function { .. } => {
                        let call: CallSiteValue = ctx.builder.build_call(fns[&self.functions[f.slice_or_name()]], &expr_values, "")?;
                        call.set_tail_call(true);
                        call.try_as_basic_value().basic().unwrap()
                    },
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Braced(_, exprs, _) => {
                let mut value = None;

                for expr in exprs {
                    value = Some(self.build_expr(ctx, expr, vars, fns)?);
                }

                value.unwrap_or_else(|| ctx.module.get_context().struct_type(&[], false).get_undef().into())
            }
            ExprRefType::Flow(l, _t, _m, _r) => {
                let val = self.build_expr(ctx, l, vars, fns)?;
                if let VariableDefinition::Match(branches, ty) = &self.references[expr.ind().0] {
                    self.build_match(ctx, vars, fns, val, branches, ty)?
                } else { unimplemented!(); }
            }
            ExprRefType::UnFlow(_, l, r) => {
                let ExprRefType::Simple(l) = l.unwrap().ty() else {unimplemented!()};

                let mut value = None;

                for expr in r {
                    value = Some(self.build_expr(ctx, expr, vars, fns)?);
                }
                let value = value.unwrap();
                if value.get_name().is_empty() {
                    value.set_name(l.slice_or_name());
                }

                vars.insert(VariableDefinition::Variable {place: expr.ind(), name: l.slice_or_name().clone(), n: 0, ty: RoType::Int }, value.into());
                value
            }
            ExprRefType::Tuple(m) => {
                let ty = self.references[expr.ind().0].ty().basic_type(ctx.module.get_context()).into_struct_type();

                let mut val = ty.get_undef();

                for (i, expr) in m.enumerate() {
                    val = ctx.builder.build_insert_value(val, self.build_expr(ctx, expr, vars, fns)?, i as u32, "")?.into_struct_value();
                };

                val.into()
            }
        })
    }

    fn build_match<'a>(&self, ctx: BuildContext<'a, '_>, vars: &mut Vars<'a>,  fns: &mut Fns<'a> , val: BasicValueEnum<'a>, branches: &[(Pattern, ExprInd)], exit_type: &RoType) -> Result<BasicValueEnum<'a>, BuilderError> {
        let val = val.into_int_value();
        let ty = exit_type.basic_type(ctx.module.get_context());

        let curr_block = ctx.builder.get_insert_block().unwrap();

        let end_block = ctx.module.get_context().append_basic_block(ctx.function, "match.end");
        ctx.builder.position_at_end(end_block);
        let phi = ctx.builder.build_phi(ty, "")?;


        let mut blocks = Vec::with_capacity(branches.len());

        let mut last_block = None;

        for (pattern, ind) in branches {
            let (_tok, _l, r) = self.ast.at(*ind).is_un_flow().unwrap();

            let block = ctx.module.get_context().insert_basic_block_after(curr_block, "match");
            ctx.builder.position_at_end(block);
            last_block = Some(block);

            let mut value = None;

            for expr in r {
                value = Some(self.build_expr(ctx, expr, vars, fns)?);
            }

            ctx.builder.build_unconditional_branch(end_block)?;

            phi.add_incoming(&[(&value.unwrap(), block)]);

            let mut subpatterns = vec![pattern];

            while let Some(subpattern) = subpatterns.pop() {
                match subpattern {
                    &Pattern::Number(n) => {
                        blocks.push((val.get_type().const_int(n, false), block));
                    }
                    Pattern::Or(sub) => {
                        subpatterns.extend(sub);
                    }
                    Pattern::Bad => {}
                }
            }
        }

        let last_block = last_block.unwrap();
        last_block.set_name("match.else");

        ctx.builder.position_at_end(curr_block);
        ctx.builder.build_switch(val, last_block, &blocks)?;

        ctx.builder.position_at_end(end_block);

        Ok(phi.as_basic_value())
    }

}

#[derive(Debug, Clone, Copy)]
struct BuildContext<'a, 'b>{
    module: &'b Module<'a>,
    function : FunctionValue<'a>,
    builder: &'b Builder<'a>,
}