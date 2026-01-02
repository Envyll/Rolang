use crate::lexer::{BracketType, SymbolType, Token, TokenType};
use crate::parser::AST;
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::semantics::{AnalyzeErr, AnalyzedAST, AnalyzerContext, FunctionDefinition, VariableDefinition};

impl AnalyzedAST {
    pub fn new(ast: AST) -> Self {
        let mut this = AnalyzerContext::default();

        let expr_ref = ast.last();
        
        this.errors.resize(expr_ref.ind().0, Default::default());
        this.references.resize(expr_ref.ind().0, Default::default());

        this.analyze_top_level_definitions(expr_ref);
        
        let AnalyzerContext{
            variables, functions, errors, references
        } = this;
        
        if !variables.is_empty() {panic!("variables stack should have been cleaned")}
        
        Self {
            ast,
            functions,
            errors,
            references,
        }
    }
}


impl AnalyzerContext {
    #[inline]
    fn do_in_variables_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.variables.push(Default::default());
        let result = f(self);
        self.variables.pop();
        result
    }

    fn analyze_top_level_definitions(&mut self, expr: ExprRef) {
        self.do_in_variables_scope( |this| {
            for def in expr.is_braced().unwrap().1 {
                if this.analyze_fn_definition(def).is_none() {
                    this.errors[def.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            
            for (_, FunctionDefinition{expr: ind, args}) in this.functions.clone() {
                let (_, _, right) = expr.ctx().at(ind).is_un_flow().unwrap();
                this.do_in_variables_scope( |this| {
                    *this.variables.last_mut().unwrap() = args;
                    for right in right {
                        this.analyze_expr(right);
                    }
                });
            }
        });
    }

    fn analyze_fn_definition(&mut self, expr: ExprRef) -> Option<()> {
        let (t, left, _right) = expr.is_un_flow()?;
        let left = left?;
        let TokenType::ColonEq = t.ty else { None? };

        let Some(mut v) = left.is_apply() else {
            self.errors[left.ind().0] = AnalyzeErr::Unimplemented;
            None?
        };
        let fst = v.next().unwrap();

        let Some(n) = fst.is_simple_default_symbol() else {
            self.errors[fst.ind().0] = AnalyzeErr::Unimplemented;
            None?
        };

        self.do_in_variables_scope( |this| {
            let mut args = <Vec<_>>::with_capacity(v.size_hint().0);

            for (n, arg) in v.enumerate() {
                let n = n as _;
                let Some(arg_n) = arg.is_simple_default_symbol() else {
                    this.errors[arg.ind().0] = AnalyzeErr::Unimplemented;
                    None?
                };
                let None = this.variables.last_mut().unwrap().insert(arg_n.clone(), VariableDefinition::Variable{
                    name: arg_n.clone(),
                    place: expr.ind(),
                    n
                }) else {
                    this.errors[arg.ind().0] = AnalyzeErr::RepeatedArg;
                    None?
                };
                args.push(arg_n.clone());
            }

            let args = this.variables.pop().unwrap();
            this.variables.push(Default::default());

            this.functions.insert(n.clone(), FunctionDefinition { args, expr: expr.ind()});
            Some(())
        })?;

        
        Some(())
    }
    
    fn analyze_expr(&mut self, expr: ExprRef) {
        //dbg!(expr.ind());
        match expr.ty() {
            ExprRefType::Empty => {}
            ExprRefType::Simple(t) => {self.analyze_var(t, expr)}
            ExprRefType::Unary(t, e) => {self.analyze_call(t, 1, expr); self.analyze_expr(e);},
            ExprRefType::Binary(l, t, r) => {self.analyze_call(t, 2, expr); self.analyze_expr(l); self.analyze_expr(r);}
            ExprRefType::Path(_, _) => {self.errors[expr.ind().0]= AnalyzeErr::Unimplemented;}
            ExprRefType::Apply(mut n) => {
                let ExprRefType::Simple(l) = n.next().unwrap().ty() else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                    for e in n {
                        self.analyze_expr(e);
                    }
                    return;
                };
                let mut cnt = 0usize;
                for e in n {
                    cnt += 1;
                    self.analyze_expr(e);
                }
                self.analyze_call(l, cnt, expr);
            }
            ExprRefType::Braced(t, n, _) => {
                self.do_in_variables_scope(|this| {
                    match t.ty {
                        TokenType::Open(BracketType::Round { with_stick: false }, _) => {
                            for e in n {
                                this.analyze_expr(e);
                            }
                        }
                        _ => {
                            this.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                            for e in n {
                                this.analyze_expr(e);
                            }
                        }
                    };

                });
            }
            ExprRefType::Flow(l, _t, m, r) => {
                self.analyze_expr(l);
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                self.do_in_variables_scope(|this| {
                    if let Some(m) = m {
                        this.errors[m.ind().0] = AnalyzeErr::Unimplemented;
                    }
                    for e in r {
                        this.analyze_expr(e);
                    }
                });
            }
            ExprRefType::UnFlow(_t, m, r) => {
                self.do_in_variables_scope(|this| {
                    for e in r {
                        this.analyze_expr(e);
                    }
                });
                
                if let Some(m) = m {
                    if let Some(Token{ty: TokenType::Symbol(s, SymbolType::Default), ..}) = m.is_simple() {
                        self.variables.last_mut().unwrap().insert(s.clone(), VariableDefinition::Variable{
                            name: s.clone(),
                            place: expr.ind(),
                            n: 0
                        });
                    } else { 
                        self.errors[m.ind().0] = AnalyzeErr::Unimplemented;
                    }
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
        }
    }
    
    fn analyze_var(&mut self, name: &Token, expr: ExprRef) {
        match &name.ty {
            TokenType::Symbol(n, SymbolType::Default) => {
                for ee in (&self.variables).into_iter().rev() {
                    if let Some(def) = ee.get(n) {
                        //dbg!(expr.ind().0);
                        self.references[expr.ind().0] = def.clone();
                        return;
                    }
                }
                //else
                //dbg!(expr.ind().0);
                self.errors[expr.ind().0] = AnalyzeErr::UnknownVar;
                self.references[expr.ind().0] = VariableDefinition::Unknown;
                return;
            }
            TokenType::Number { start_pos: 0, dot_pos: None, exp_pos: None, suf_pos: None, .. } => {
                if let Ok(n) = u64::from_str_radix(name.slice.as_str(), 10) {
                    self.references[expr.ind().0] = VariableDefinition::Number(n);
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            _ => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
        }
        
    }
    
    fn analyze_call(&mut self, name: &Token, cnt: usize, expr: ExprRef) {
        match (&name.ty, name.slice.as_str()) {
            (TokenType::Symbol(n, SymbolType::Default), _) => {
                let Some(c) = self.functions.get(n) else {
                    self.errors[expr.ind().0] = AnalyzeErr::UnknownVar;
                    self.references[expr.ind().0] = VariableDefinition::Unknown;
                    return;
                };
                if c.args.len() != cnt {
                    self.errors[expr.ind().0] = AnalyzeErr::BadFnCall;
                    return;
                }
                self.references[expr.ind().0] = VariableDefinition::Function(n.clone());
            }
            (TokenType::Operator, "+" | "-" | "*") => {
                self.references[expr.ind().0] = VariableDefinition::Special(name.slice.clone());
            }
            _ => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
        }
    }
}