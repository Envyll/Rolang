use std::ops::Deref;
use crate::lexer::{Token, };
use crate::parser::{ASTExpr, ExprInd, AST};
use ASTExpr::*;

#[derive(Copy, Clone)]
pub struct ExprRef<'ctx> {
    expr: &'ctx ASTExpr,
    ind: ExprInd,
    ctx: &'ctx AST,
}

impl<'ctx> Deref for ExprRef<'ctx> {
    type Target = ASTExpr;
    fn deref(&self) -> &'ctx Self::Target {
        self.expr
    }
}

impl<'ctx> AsRef<ASTExpr> for ExprRef<'ctx> {
    fn as_ref(&self) -> &'ctx ASTExpr {
        self.expr
    }
}

impl AST{    
    pub fn at(&self, ind: ExprInd) -> ExprRef<'_> {
        ExprRef {
            ind,
            ctx: self,
            expr: &self[ind]
        }
    }
    
    pub fn last(&self) -> ExprRef<'_> {
        self.at(ExprInd(self.expressions.len() - 1))
    }
}

impl<'ctx> ExprRef<'ctx> {    
    pub fn ind(self) -> ExprInd {
        self.ind
    } 
    pub fn ctx(self) -> &'ctx AST {
        self.ctx
    }
    pub fn expr(self) -> &'ctx ASTExpr {
        self.expr
    }
    
    pub fn is_simple(self) -> Option<&'ctx Token> {
        let &Simple(token) = self.expr else { None? };
        Some(&self.ctx[token])
    }
    
    pub fn is_unary(self) -> Option<(&'ctx Token, Self)> {
        let &Unary(token, expr) = self.expr else { None? };
        Some((&self.ctx[token], self.ctx.at(expr)))
    }
    
    pub fn is_simple_or_unary(self) -> Option<&'ctx Token> {
        let &(Unary(token, _) | Simple(token)) = self.expr else { None? };
        Some(&self.ctx[token])
    }
    
    pub fn is_binary(self) -> Option<(Self, &'ctx Token, Self)> {
        let &Binary(lhs, token, rhs) = self.expr else { None? };
        Some((self.ctx.at(lhs), &self.ctx[token], self.ctx.at(rhs)))
    }
    
    pub fn is_apply(self) -> Option<Self> {
        let Apply(_) = self.expr else { None? };
        Some(self)
    }
    
    
}

impl<'ctx> IntoIterator for ExprRef<'ctx> {
    type Item = Self;
    type IntoIter = Box<dyn Iterator<Item=Self> + 'ctx>;

    fn into_iter(self) -> Self::IntoIter {
        match self.expr { 
            Empty | Simple(_) => Box::new(None.into_iter()),
            &Unary(_, r) => Box::new(Some(self.ctx.at(r)).into_iter()),
            &Binary(l, _, r) => Box::new(Some(l).into_iter().chain(Some(r)).map(|e| self.ctx.at(e))),
            &Path(ref l, ref n) => Box::new(Some(l).into_iter().chain(n.iter()).map(|e| self.ctx.at(*e))),
            &Apply(ref n) | &Braced(_, ref n, _) => Box::new(n.into_iter().map(|e| self.ctx.at(*e))),
            &Flow(l, _, m, ref r) => Box::new(Some(l).into_iter().chain(m.into_iter()).chain(r.iter().map(|e| *e)).map(|e| self.ctx.at(e)) ),
            &UnFlow(_, m, ref r) => Box::new(m.into_iter().chain(r.iter().map(|&e|e)).map(|e| self.ctx.at(e))),
        }
    }
}