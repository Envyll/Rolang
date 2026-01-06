use crate::lexer::{BracketType, SymbolType, Token, TokenType};
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::parser::AST;
use crate::semantics::types::RoType;
use crate::semantics::{AnalyzeErr, AnalyzedAST, AnalyzerContext, FunctionDefinition, VariableDefinition};

mod analyze_match;

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
                if this.analyze_fn_declaration(def).is_none() {
                    this.errors[def.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            
            for (_, def) in this.functions.clone() {
                this.analyze_fn_definition(&def, expr.ctx());
            }
        });
    }

    fn analyze_fn_declaration(&mut self, expr: ExprRef) -> Option<()> {
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
                    n,
                    ty : RoType::Int,
                }) else {
                    this.errors[arg.ind().0] = AnalyzeErr::RepeatedArg;
                    None?
                };
                args.push(arg_n.clone());
            }

            let args = this.variables.pop().unwrap();
            this.variables.push(Default::default());

            this.functions.insert(n.clone(), FunctionDefinition { args, expr: expr.ind(), return_type: Default::default() });
            Some(())
        })?;

        Some(())
    }

    fn analyze_fn_definition<'a>(&mut self, def: &'a FunctionDefinition, ctx: &AST) -> &'a RoType {
        def.return_type.get_or_init(|| {
            let (_, _, right) = ctx.at(def.expr).is_un_flow().unwrap();
            let mut last = RoType::unit();

            self.do_in_variables_scope(|this| {
                *this.variables.last_mut().unwrap() = def.args.clone();
                for right in right {
                    last = this.analyze_expr(right);
                }
            });
            last
        })
    }
    
    fn analyze_expr(&mut self, expr: ExprRef) -> RoType {
        //dbg!(expr.ind());
        match expr.ty() {
            ExprRefType::Empty => {RoType::Never}
            ExprRefType::Simple(t) => {self.analyze_var(t, expr)}
            ExprRefType::Unary(t, e) => {
                let rt = self.analyze_expr(e);
                self.analyze_call(t, &[rt], expr)
            },
            ExprRefType::Binary(l, t, r) => {
                let lt = self.analyze_expr(l);
                let rt = self.analyze_expr(r);
                self.analyze_call(t, &[lt, rt], expr)
            }
            ExprRefType::Path(_, _) => {self.errors[expr.ind().0]= AnalyzeErr::Unimplemented; RoType::Never}
            ExprRefType::Apply(n) => {
                self.analyze_apply(expr, n)
            }
            ExprRefType::Braced(t, n, _) => {
                self.do_in_variables_scope(|this| {
                    match t.ty {
                        TokenType::Open(BracketType::Round { with_stick: false }, _) => {
                            let mut ty = RoType::unit();

                            for e in n {
                                ty = this.analyze_expr(e);
                            }
                            ty
                        }
                        _ => {
                            this.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                            for e in n {
                                this.analyze_expr(e);
                            }
                            RoType::Never
                        }
                    }

                })
            }
            ExprRefType::Flow(l, t, m, r) => {
                self.analyze_expr(l);

                if let TokenType::Keyword(kw) = &t.ty && kw == "match" && m.is_none() {
                    self.analyze_match(expr, r)
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                    self.do_in_variables_scope(|this| {
                        if let Some(m) = m {
                            this.errors[m.ind().0] = AnalyzeErr::Unimplemented;
                        }
                        for e in r {
                            this.analyze_expr(e);
                        }
                        RoType::Never
                    })
                }
            }
            ExprRefType::UnFlow(t, m, r) => {
                if !matches!(t.ty, TokenType::ColonEq) {self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;}

                let mut ty = RoType::unit();

                self.do_in_variables_scope(|this| {
                    for e in r {
                        ty = this.analyze_expr(e);
                    }
                });
                
                if let Some(m) = m {
                    if let Some(Token{ty: TokenType::Symbol(s, SymbolType::Default), ..}) = m.is_simple() {
                        self.variables.last_mut().unwrap().insert(s.clone(), VariableDefinition::Variable{
                            name: s.clone(),
                            place: expr.ind(),
                            n: 0,
                            ty
                        });
                        RoType::unit()
                    } else { 
                        self.errors[m.ind().0] = AnalyzeErr::Unimplemented;
                        RoType::Never
                    }
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                    RoType::Never
                }
            }
            ExprRefType::Tuple(n) => {
                let mut types = Vec::new();
                for e in n {
                    types.push(self.analyze_expr(e));
                };
                self.references[expr.ind().0] = VariableDefinition::Tuple(types.clone());
                RoType::Tuple(types)
            }
        }
    }

    fn analyze_number_literal(&mut self, name: &Token, expr: ExprRef) -> Option<u64> {

        if let Ok(n) = u64::from_str_radix(name.slice.as_str(), 10) {
            self.references[expr.ind().0] = VariableDefinition::Number(n);
            Some(n)
        } else {
            self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            None
        }

    }

    fn analyze_var(&mut self, name: &Token, expr: ExprRef) -> RoType {
        match &name.ty {
            TokenType::Symbol(n, SymbolType::Default) => {
                for ee in (&self.variables).into_iter().rev() {
                    if let Some(def) = ee.get(n) {
                        //dbg!(expr.ind().0);
                        self.references[expr.ind().0] = def.clone();
                        return def.ty();
                    }
                }
                //else
                //dbg!(expr.ind().0);
                self.errors[expr.ind().0] = AnalyzeErr::UnknownVar;
                self.references[expr.ind().0] = VariableDefinition::Unknown;
                RoType::Never
            }
            TokenType::Number { .. } => {
                self.analyze_number_literal(name, expr);
                RoType::Int
            }
            _ => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                RoType::Never
            }
        }
        
    }
    
    fn analyze_call(&mut self, name: &Token, args: &[RoType], expr: ExprRef) -> RoType {
        match (&name.ty, name.slice.as_str()) {
            (TokenType::Symbol(n, SymbolType::Default), _) => {
                let Some(c) = self.functions.get(n) else {
                    self.errors[expr.ind().0] = AnalyzeErr::UnknownVar;
                    self.references[expr.ind().0] = VariableDefinition::Unknown;
                    return RoType::Never;
                };
                if c.args.len() != args.len() {
                    self.errors[expr.ind().0] = AnalyzeErr::BadFnCall;
                    return RoType::Never;
                }
                self.references[expr.ind().0] = VariableDefinition::Function(n.clone());
                self.analyze_fn_definition(&c.clone(), expr.ctx()).clone()
            }
            (TokenType::Operator, "+" | "-" | "*") => {
                self.references[expr.ind().0] = VariableDefinition::Special(name.slice.clone());
                RoType::Int
            }
            (TokenType::Number {..}, _) => {
                let Some(n) = self.analyze_number_literal(name, expr) else {
                    return RoType::Never;
                };
                let n = n as u32;
                self.references[expr.ind().0] = VariableDefinition::Member(n);
                let [RoType::Tuple(types)] = args else {
                    self.errors[expr.ind().0] = AnalyzeErr::BadFnCall;
                    return RoType::Never;
                };
                let Some(t) = types.get(n as usize) else {
                    self.errors[expr.ind().0] = AnalyzeErr::BadFnCall;
                    return RoType::Never;
                };
                t.clone()
            }
            _ => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                RoType::Never
            }
        }
    }

    fn analyze_apply<'a>(&mut self, expr: ExprRef, mut n: impl Iterator<Item=ExprRef<'a>>) -> RoType {
        let ExprRefType::Simple(l) = n.next().unwrap().ty() else {
            self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            for e in n {
                self.analyze_expr(e);
            }
            return RoType::Never;
        };
        let mut cnt = vec![];
        for e in n {
            cnt.push(self.analyze_expr(e));
        }
        self.analyze_call(l, &cnt, expr)
    }


}