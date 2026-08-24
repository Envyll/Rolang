use crate::AResult;
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::semantics::ro_type::Replacements;
use crate::semantics::{AnalyzedAST, BindingReference, ExpressionDescription, FnInstanceDescr};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue};
use itertools::{Itertools, izip};
use std::collections::HashMap;

pub type Vars<'a> = HashMap<BindingReference, BasicValueEnum<'a>>;
pub type Fns<'a> = HashMap<FnInstanceDescr, FunctionValue<'a>>;

#[derive(Debug)]
pub struct BuildContext<'a, 'b> {
    pub module: &'b Module<'a>,
    pub builder: Builder<'a>,
    pub vars: Vars<'a>,
    pub fns: &'b mut Fns<'a>,

    pub repl: &'b mut Replacements,
    pub unreachable: BasicBlock<'a>,
}

impl AnalyzedAST {
    pub fn build_module(&self, module: &Module) -> AResult {
        let ref mut fns = HashMap::new();

        //dbg!(self.instances.borrow());

        for (descr, def) in self.instances.borrow().iter() {
            let out_ty = def.ty.basic_type(module.get_context());
            let in_ty = descr
                .args
                .iter()
                .map(|arg| arg.basic_type(module.get_context()).into())
                .collect_vec();

            let f_ty = BasicType::fn_type(&out_ty, &in_ty, false);

            let f = module.add_function(&descr.name, f_ty, Some(Linkage::External));
            //f.set_call_conventions(18);

            fns.insert(descr.clone(), f.into());
        }

        //module.print_to_stderr();

        for (def, res) in self.instances.borrow_mut().iter_mut() {
            let builder = module.get_context().create_builder();

            let entry = module.get_context().append_basic_block(fns[def], "");

            let unreachable = module
                .get_context()
                .append_basic_block(fns[def], "unreachable");
            builder.position_at_end(unreachable);
            builder.build_unreachable()?;

            builder.position_at_end(entry);

            let ctx = BuildContext {
                module,
                builder,
                vars: Default::default(),
                fns,
                repl: &mut res.replacements,
                unreachable,
            };
            self.build_fn(ctx, def)?;
        }

        //module.print_to_stderr();

        module.verify().map_err(|err| anyhow::anyhow!("{err}"))?;

        Ok(())
    }

    fn build_fn(&self, mut ctx: BuildContext, def: &FnInstanceDescr) -> AResult {
        let f = ctx.fns[def];
        let descr = &self.functions[&def.name];

        let ExprRefType::UnFlow(_, _, right) = self.ast.at(self.functions[&def.name].expr).ty()
        else {
            unreachable!()
        };

        let mut value = None;

        for (i, arg, patt, par) in izip!(
            0..,
            def.args.iter(),
            &descr.arg_patterns,
            f.get_param_iter()
        ) {
            self.build_let(&mut ctx, par, patt, arg, &format!("arg{i}"))?
        }

        for expr in right {
            value = self.build_expr(&mut ctx, expr)?.into();
        }
        ctx.builder.build_return(Some(&value.unwrap()))?;
        Ok(())
    }

    pub(super) fn build_expr<'a>(
        &self,
        ctx: &mut BuildContext<'a, '_>,
        expr: ExprRef,
    ) -> AResult<BasicValueEnum<'a>> {
        Ok(match expr.ty() {
            ExprRefType::Empty => {
                unimplemented!()
            }
            ExprRefType::Simple(_) => match &self.references[expr.ind().0] {
                ExpressionDescription::Number(n) => ctx
                    .module
                    .get_context()
                    .i64_type()
                    .const_int(*n, false)
                    .into(),
                ExpressionDescription::Variable(def) => {
                    //dbg!(def);
                    let vars = &ctx.vars;
                    *vars
                        .get(&def)
                        .expect(&format!("Cannot find {def:#?} (known are {vars:#?})"))
                }
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
            ExprRefType::Apply(exprs) => {
                let mut expr_values: Vec<BasicMetadataValueEnum> = vec![];

                for expr in exprs.skip(1) {
                    expr_values.push(self.build_expr(ctx, expr)?.into());
                }

                match &self.references[expr.ind().0] {
                    &ExpressionDescription::Member(n) => {
                        let value = expr_values.first().unwrap().into_struct_value();
                        ctx.builder.build_extract_value(
                            value,
                            n,
                            &format!("{}.{}", value.get_name().to_str()?, n),
                        )?
                    }
                    ExpressionDescription::Function { name, input, .. } => {
                        let call: CallSiteValue = ctx.builder.build_call(
                            ctx.fns[&FnInstanceDescr {
                                name: name.clone(),
                                args: input.iter().map(|n| n.deep_clone(ctx.repl)).collect(),
                            }],
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
                if let ExpressionDescription::Match {
                    branches,
                    patterns,
                    ty,
                    pred_ty,
                } = &self.references[expr.ind().0]
                {
                    let name = val.get_name().to_str()?;
                    let ty = &ty.deep_clone(ctx.repl);
                    self.build_match(
                        ctx,
                        val,
                        &branches,
                        &patterns,
                        ty,
                        pred_ty,
                        &format!("{name}.match"),
                    )?
                } else {
                    unimplemented!();
                }
            }
            ExprRefType::UnFlow(_, _, r) => {
                let ExpressionDescription::Let { patterns, pred_ty } =
                    &self.references[expr.ind().0]
                else {
                    unimplemented!()
                };

                let mut value = None;

                for expr in r {
                    value = Some(self.build_expr(ctx, expr)?);
                }
                let value = value.unwrap();
                let name = value.get_name().to_str()?;

                //let pred_ty = &pred_ty.deep_clone(ctx.repl);

                self.build_let(ctx, value, patterns, pred_ty, &format!("{name}.let"))?;
                value
            }
            ExprRefType::Tuple(m) => {
                let len = m.len();
                let mut values = Vec::with_capacity(len);
                let mut types = Vec::with_capacity(len);
                for expr in m {
                    let val = self.build_expr(ctx, expr)?;
                    values.push(val);
                    types.push(val.get_type());
                }

                let val_ty = ctx.module.get_context().struct_type(&types, false);
                let mut val = val_ty.get_undef();

                for (i, &value) in values.iter().enumerate() {
                    val = ctx
                        .builder
                        .build_insert_value(
                            val,
                            value,
                            i as u32,
                            &format!("tuple{}.build.{}", len, i),
                        )?
                        .into_struct_value();
                }

                val.into()
            }
            ExprRefType::Either(_) => {
                unimplemented!()
            }
        })
    }
}
