use std::collections::HashMap;
use inkwell::builder::{Builder, BuilderError};
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::values::{AnyValueEnum,  IntValue};
use maplit::hashmap;
use crate::ImStr;
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::semantics::{AnalyzedAST, FunctionDefinition, VariableDefinition};

impl AnalyzedAST {
    pub fn build_module(&self, module: &Module) -> Result<(), BuilderError> {

        let mut vars = hashmap! {};
        
        
        for (name, def) in &self.functions {
            let ty = module.get_context().i64_type();
            let f_ty = ty.fn_type(&vec![ty.into(); def.args.len()], false);

            let f = module.add_function(name.as_str(), f_ty, Some(Linkage::External));
            
            vars.insert(VariableDefinition::Function(name.clone()), f.into());
        }
        for (name, def) in &self.functions {
            self.build_fn(module, name, def, &mut vars)?;
        }

        module.print_to_stderr();
        
        module.verify().unwrap();

        Ok(())
    }

    fn build_fn<'a>(&self, module: &Module<'a>, name: &ImStr, def: &FunctionDefinition, vars: &mut HashMap<VariableDefinition, AnyValueEnum<'a>>) -> Result<(), BuilderError> {
        let f = vars[&VariableDefinition::Function(name.clone())].into_function_value();
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
            value = self.build_expr(module, &builder, expr, vars)?.into();
        }
        builder.build_return(Some(&value.unwrap()))?;
        Ok(())
    }

    fn build_expr<'a>(&self, module: &Module<'a>,  builder: &Builder<'a>, expr: ExprRef, vars: &mut HashMap<VariableDefinition, AnyValueEnum<'a>>) -> Result<IntValue<'a>, BuilderError> {
        Ok(match expr.ty() {
            ExprRefType::Empty => { unimplemented!() }
            ExprRefType::Simple(_) => {
                match &self.references[expr.ind().0] {
                    VariableDefinition::Number(n) => module.get_context().i64_type().const_int(*n, false),
                    def @ VariableDefinition::Variable { .. } => vars[def].into_int_value(),
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Unary(op, r) => {
                match op.slice.as_str() {
                    "+" | "*" => {
                        self.build_expr(module, builder, r, vars)?
                    }
                    "-" => {
                        let r_value = self.build_expr(module, builder, r, vars)?;
                        builder.build_int_neg(r_value, "")?
                    }
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Binary(l, op, r) => {
                let l_value = self.build_expr(module, builder, l, vars)?;
                let r_value = self.build_expr(module, builder, r, vars)?;
                match op.slice.as_str() {
                    "+" => {
                        builder.build_int_add(l_value, r_value, "")?
                    }
                    "*" => {
                        builder.build_int_mul(l_value, r_value, "")?
                    }
                    "-" => {
                        builder.build_int_sub(l_value, r_value, "")?
                    }
                    _ => unimplemented!(),
                }
            }
            ExprRefType::Path(_, _) => {unimplemented!()}
            ExprRefType::Apply(mut exprs) => {
                let ExprRefType::Simple(f) = exprs.next().unwrap().ty() else {unimplemented!()};
                let mut expr_values = vec![];

                for expr in exprs {
                    expr_values.push(self.build_expr(module, builder, expr, vars)?.into());
                };
                builder.build_call(vars[&VariableDefinition::Function(f.slice_or_name().clone())].into_function_value(), &expr_values, "")?
                    .try_as_basic_value().basic().unwrap().into_int_value()
            }
            ExprRefType::Braced(_, exprs, _) => {
                let mut value = None;

                for expr in exprs {
                    value = Some(self.build_expr(module, builder, expr, vars)?);
                }

                // else unimplemented
                value.unwrap()
            }
            ExprRefType::Flow(_, _, _, _) => {unimplemented!()}
            ExprRefType::UnFlow(_, l, r) => {
                let ExprRefType::Simple(l) = l.unwrap().ty() else {unimplemented!()};

                let mut value = None;

                for expr in r {
                    value = Some(self.build_expr(module, builder, expr, vars)?);
                }
                let value : IntValue = value.unwrap();
                if value.get_name().is_empty() {
                    value.set_name(l.slice_or_name());
                }

                vars.insert(VariableDefinition::Variable {place: expr.ind(), name: l.slice_or_name().clone(), n: 0}, value.into());
                value
            }
        })
    }


}