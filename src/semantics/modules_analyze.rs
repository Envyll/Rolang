use crate::parser::ast_expr_traits::ExprRef;
use crate::parser::AST;
use crate::semantics::constants_values::ConstantValue;
use crate::semantics::modules::{ConstantInfo, Module, ModuleInd};
use crate::semantics::{FileTree, ModulesTable, Place};
use im::hashmap;
use std::cell::RefCell;

pub fn modules_analyze(this: &FileTree) -> ModulesTable {
    let mut result = ModulesTable::default();

    for (n, ast) in &this.0{
        result.0.insert(
            ModuleInd(n.clone()),
            Module{
                this: ConstantInfo {
                    name: n.last().map(Clone::clone).unwrap_or_default(),
                    declaration: Place {
                        file: n.clone(),
                        expr: ast.root(),
                    },
                    definitions: vec![],
                    dependencies: hashmap!{},
                    value: RefCell::new(maplit::hashmap!{hashmap!{} => ConstantValue::Type(None).into()}),
                },
                params: hashmap!{},
                constants: hashmap!{},
                struct_type: None,
                is_public: false,
            }
        );
    }

    for module in result.0.values_mut() {
        analyze_module_for_type_declarations(&this.0[&module.this.declaration.file], module)
    }
    
    todo!();
}

#[allow(unused_variables)]
fn analyze_module_for_type_declarations(ast: &AST, root: &mut Module) {
    
    let mut exprs_stack = vec![];

    exprs_stack.push((ast.last().is_apply().unwrap().into_iter(), root));

    while let Some((exprs, module)) = exprs_stack.pop() {
        'exprs: for expr in exprs {
            let Some((app, body)) = is_type_declaration(expr) else { continue 'exprs };
            todo!()
        }
    }
    
    todo!()
}

fn is_type_declaration(expr: ExprRef) -> Option<(ExprRef, ExprRef)> {
    let (mut lhs, _, rhs) = expr.is_binary().take_if(|t| t.1.is_op(":="))?;
    while let Some((l, _, _)) = lhs.is_binary().take_if(|t| t.1.is_kw("when")) {
        lhs = l;
    }
    lhs.is_binary()?.2.is_simple_or_unary().take_if(|t| t.is_kw("type"))?;
    Some((lhs, rhs))
}

