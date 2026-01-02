use std::ops::Deref;
use crate::lexer::{SymbolType, Token, TokenType};
use crate::parser::{ASTExpr, ExprInd, ExprVec, AST};
use ASTExpr::*;
use crate::ImStr;

#[derive(Copy, Clone, Debug)]
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
    
    pub fn is_simple_default_symbol(self) -> Option<&'ctx ImStr> {
        let &Simple(token) = self.expr else { None? };
        let Token{ty: TokenType::Symbol(n, SymbolType::Default), ..} = &self.ctx[token] else { None? };
        Some(n)
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
    
    pub fn is_flow(self) -> Option<(Self, &'ctx Token, Option<Self>, impl Iterator<Item = Self> )> {
        let &Flow(lhs, token, mhs, ref rhs) = self.expr else { None? };
        Some((self.ctx.at(lhs), &self.ctx[token], mhs.map(|mhs| self.ctx.at(mhs)), rhs.into_iter().map(|e| self.ctx.at(*e))))
    }
    
    pub fn is_un_flow(self) -> Option<(&'ctx Token, Option<Self>, impl Iterator<Item = Self>)> {
        let &UnFlow(token, mhs, ref rhs) = self.expr else { None? };
        Some((&self.ctx[token], mhs.map(|mhs| self.ctx.at(mhs)), rhs.into_iter().map(|e| self.ctx.at(*e))))
    }
    
    pub fn is_apply(self) -> Option<impl Iterator<Item = Self>> {
        let Apply(n) = self.expr else { None? };

        Some(n.into_iter().map(|e| self.ctx.at(*e)))
    }

    pub fn is_braced(self) -> Option<(&'ctx Token, impl Iterator<Item = Self>, &'ctx Token,)> {
        let &Braced(l, ref n, r) = self.expr else { None? };

        Some((&self.ctx[l], n.into_iter().map(|e| self.ctx.at(*e)), &self.ctx[r]))
    }
    
    pub fn ty(self) -> ExprRefType<'ctx, impl Iterator<Item=ExprRef<'ctx>> + 'ctx> {
        let at = |&e| self.ctx.at(e);
        let atm = |e: &Option<_>| e.map(|e| self.ctx.at(e));
        let att = |ee: &'ctx ExprVec| ee.into_iter().map(|&e| self.ctx.at(e));
        let tok = |&i| &self.ctx[i];
        
        match &self.expr {
            Empty => ExprRefType::Empty,
            Simple(t) => ExprRefType::Simple(tok(t)),
            Path(l, r) => ExprRefType::Path(at(l), att(r)),
            Apply(n) => ExprRefType::Apply(att(n)),
            Unary(t, r) => ExprRefType::Unary(tok(t), at(r)),
            Binary(l, t, r) => ExprRefType::Binary(at(l), tok(t), at(r)),
            Braced(l, m, r) => ExprRefType::Braced(tok(l), att(m), tok(r)),
            Flow(l, t, m, r) => ExprRefType::Flow(at(l), tok(t), atm(m), att(r)),
            UnFlow(t, m, r) => ExprRefType::UnFlow(tok(t), atm(m), att(r)),
        }
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

pub enum ExprRefType<'ctx, Iter: Iterator<Item=ExprRef<'ctx>> + 'ctx> {
    Empty,
    Simple(&'ctx Token),
    Unary(&'ctx Token, ExprRef<'ctx>),
    Binary(ExprRef<'ctx>, &'ctx Token, ExprRef<'ctx>),
    Path(ExprRef<'ctx>, Iter),
    Apply(Iter),
    Braced(&'ctx Token, Iter, &'ctx Token, ),
    Flow(ExprRef<'ctx>, &'ctx Token, Option<ExprRef<'ctx>>, Iter),
    UnFlow(&'ctx Token, Option<ExprRef<'ctx>>, Iter),
}