use rolang::lexer::TokenIterator;
use rolang::parser::visualize::visualize_ast;
use rolang::parser::{AST, ASTExpr, ExprInd};

fn parse(str: &str) -> AST {
    TokenIterator::new(str).normalize().parse()
}

macro_rules! test_nodes [
    [$($x:tt)*] => {test_parse_node(vec![$(Box::new(test_nodes!$x)),*])};
];

fn test_parse(str: &str, f: impl FnOnce(&AST, ExprInd)) {
    let ast = &parse(str);
    println!("{}", visualize_ast(ast));
    //let ASTExpr::Braced(..) = &ast.expressions.last().unwrap().1 else { panic!() };
    f(ast, (ast.expressions.len() - 1).into())
}

#[allow(unused_variables)]
fn test_parse_node(fns: Vec<Box<dyn FnOnce(&AST, ExprInd)>>) -> impl FnOnce(&AST, ExprInd) {
    move |ast, expr| {
        let len = fns.len();
        match ast[expr] {
            ASTExpr::Empty => {
                assert_eq!(len, 0, "fail at empty")
            }
            ASTExpr::Simple(tok) => {
                let t = &ast[tok].slice;
                assert_eq!(len, 0, "fail at simple({t:})")
            }
            ASTExpr::Path(root, ref args) => {
                assert_eq!(len, 1 + args.len(), "fail at path");
                let mut fns = fns.into_iter();
                fns.next().unwrap()(ast, root);
                for (i, f) in fns.enumerate() {
                    f(&ast, args[i]);
                }
            }
            ASTExpr::Apply(ref args) => {
                assert_eq!(len, args.len(), "fail at apply");
                for (f, arg) in fns.into_iter().zip(args.iter()) {
                    f(&ast, *arg);
                }
            }
            ASTExpr::Unary(tok, expr) => {
                let t = &ast[tok].slice;
                assert_eq!(len, 1, "fail at unary({t:})");
                fns.into_iter().next().unwrap()(ast, expr);
            }
            ASTExpr::Binary(lhs, tok, rhs) => {
                let t = &ast[tok].slice;
                assert_eq!(len, 2, "fail at binary({t:})");
                let mut fns = fns.into_iter();
                fns.next().unwrap()(ast, lhs);
                fns.next().unwrap()(ast, rhs);
            }
            ASTExpr::Braced(ltok, ref exprs, rtok) => {
                let l = &ast[ltok].slice;
                let r = &ast[rtok].slice;
                assert_eq!(len, exprs.len(), "fail at braced({l:}{r:})");
                for (i, f) in fns.into_iter().enumerate() {
                    f(&ast, exprs[i]);
                }
            }
            ASTExpr::Flow(lhs, tok, mhs, ref rhs) => {
                assert_eq!(
                    len,
                    rhs.len() + 1 + mhs.is_some() as usize,
                    "fail at flow({})",
                    &ast[tok].slice
                );
                let mut fns = fns.into_iter();
                fns.next().unwrap()(ast, lhs);
                if let Some(mhs) = mhs {
                    fns.next().unwrap()(ast, mhs);
                }
                for (i, f) in fns.enumerate() {
                    f(&ast, rhs[i]);
                }
            }
            ASTExpr::UnFlow(tok, mhs, ref rhs) => {
                assert_eq!(
                    len,
                    rhs.len() + mhs.is_some() as usize,
                    "fail at un_flow({})",
                    &ast[tok].slice
                );
                let mut fns = fns.into_iter();
                if let Some(mhs) = mhs {
                    fns.next().unwrap()(ast, mhs);
                }
                for (i, f) in fns.enumerate() {
                    f(&ast, rhs[i]);
                }
            }
            ASTExpr::Tuple(ref exprs) => {
                assert_eq!(len, exprs.len(), "fail at tuple({})", exprs.len());
                for (i, f) in fns.into_iter().enumerate() {
                    f(&ast, exprs[i]);
                }
            }
            ASTExpr::Either(ref exprs) => {
                assert_eq!(len, exprs.len(), "fail at either({})", exprs.len());
                for (i, f) in fns.into_iter().enumerate() {
                    f(&ast, exprs[i]);
                }
            }
        }
    }
}

#[test]
fn test_op() {
    test_parse(
        "3 = 3",
        test_nodes![[
            []
            []
        ]],
    )
}

#[test]
fn test_ops() {
    test_parse(
        "- 0 ** 0 + 1 ** 2 ** 3 + 1 = 3",
        test_nodes![[ // =
             [ //+
                [ //+
                    [[ //- **
                        [] //0
                        [] //0
                    ]]
                    [ //**
                        [] //1
                        [ //**
                            [] //2
                            [] //3
                        ]
                    ]
                ]
                [] //1
            ]
            [] //3
        ]],
    );
}

#[test]
#[ignore] // fix/me: will change
fn test_def() {
    test_parse(
        r#"
        pub some_mod T  
        when: 
            T.Default
        type{} :=
            pub my_type type[priv T]~ :=
            pub my_static := my_type::[||] mut@
    "#,
        test_nodes![
            {
                {{
                    ([[]][])
                    ([][[]])
                }}
                {{[[]] ([[[]]][])}}
                {[[]] (([][]) [[]])}
            }
        ],
    )
}

#[test]
fn test_flow() {
    test_parse(
        "
        [] then n:{
            n
        }
        u then m:
            m
        a then x: {} then {}
        a then then {}
        a or or {} then {} or {}

        a then b : c
        a then: c
        a then c
        a then {c}

        a match {}
        a match
            a: {}
            b: {}
        a b := a match
            a : a
    ",
        test_nodes![
           {[][][]}
           {[][][]}
           {{[][]}}
           {[]{}}
           {{{[]{}}}}

           {[][][]}
           {[][]}
           {[][]}
           {[][]}

           {[]}
           {[] {[]} {[]}}
           {{[] []} {[] {[] []}}}
        ],
    )
}

#[test]
fn test_tuples() {
    test_parse(
        "
        a, b  | c, d..e, f, | g, h, i,
    ",
        test_nodes![
            [([][]) ([][[][]][]) ([][][]) ]
        ],
    )
}
