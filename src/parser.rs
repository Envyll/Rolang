use crate::lexer::Token;

mod parser_old;
mod priority;
// … → >1
// ? → ⩽1
// ?… → any
// a/b/… → smth like 'a', 'b'
// 'arg → any pattern expression

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TokenInd(pub usize);

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExprInd(pub usize);

impl From<usize> for TokenInd {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/*#[derive(Debug, Clone, Copy)]
pub enum DefinitionObjectType {
    // Uses the result of body execution as definition
    // Example:
    // b := Some 0
    // (Some a) := b
    // a + b
    Val,
    // Uses the body itself as definition
    // Example:
    // f (Some a) := a * 2
    // f (Some 0)
    Fn,
    // Uses the arguments and sub-definitions in body as definition
    // Example:
    // T a type := (this.f b := a + b)
    // x := T 0
    // x.a
    // g x 1
    Type,
    // Uses sub-definitions as definition
    // Example:
    // TT T trait := ((this as T).g := T.a)
    // t := SomeT()
    // t.TT::g
    Trait,
}

#[derive(Clone, Debug)]
pub struct Definition{
    // ('pat)/<name 'args?… type/trait/mods?> :/:=/::= body?

    pub def: ExprInd,
    pub eq: TokenInd,
    pub body: Option<ExprInd>,
    pub ty: DefinitionObjectType,
}*/

#[derive(Clone, Debug)]
pub enum ASTExpr {
    // abc
    Simple(TokenInd),      
    // abc::def abc::*** abc::def::ghi (A B)::C expression::('a := A; 'B := B; ?…)                         
    Path(Vec<ExprInd>), 
    // (abc def…)
    Apply(Vec<ExprInd>), 
    // (abc <| def) (abc := def) (abc;def) (..) (+abc) …
    Op(Option<ExprInd>, TokenInd, Option<ExprInd>), 
    // abc, def
    Tuple(Vec<ExprInd>),                            
    // 「abc; def」{abc} ⟨abc⟩ () {|abc|} …
    Braced(TokenInd, Vec<ExprInd>, TokenInd),       
}

#[derive(Clone, Debug)]
pub struct AST {
    tokens: Vec<Token>,
    expressions: Vec<ASTExpr>,
    errors: Vec<(ExprInd, ASTErr)>,
}

#[derive(Clone, Debug, Copy, Default)]
pub enum ParseErr {
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Copy)]
pub enum ASTErr {
    Unclosed,
}
