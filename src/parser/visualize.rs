use crate::parser::{AST, ASTExpr, ExprInd};

// this part is (partially) ai-created

pub fn visualize_ast_full(ast: &AST) -> String {
    let mut output = String::new();

    for i in 0..ast.expressions.len() {
        let expr_ind = ExprInd(i);

        visualize_expr(ast, expr_ind, 1, &mut output);

        output += "│ \n";
    }

    output
}

pub fn visualize_ast(ast: &AST) -> String {
    let mut output = String::new();

    let expr_ind = (ast.expressions.len() - 1).into();
    visualize_expr(ast, expr_ind, 0, &mut output);

    output
}

fn visualize_expr(ast: &AST, expr: ExprInd, depth: usize, out: &mut String) {
    let indent = if depth == 0 {
        *out += &format!("{expr}\n");
        "".to_string()
    } else {
        let indent = "│ ".repeat(depth - 1);
        *out += &format!("{indent}┝━{expr}\n");
        indent + "│ "
    };
    if let Some(errs) = ast.errors.get(expr) {
        for err in errs {
            *out += &format!("{indent}⚠️  Error: {}\n", err);
        }
    }
    match ast[expr] {
        ASTExpr::Simple(lit) => {
            let lit = &ast.tokens[lit.0].slice;
            out.push_str(&format!("{indent}Simple: {lit:?}\n"));
        }
        ASTExpr::Binary(lhs, op, rhs) => {
            let op = &ast.tokens[op.0].slice;
            out.push_str(&format!("{indent}Binary: {op:?}\n"));
            visualize_expr(&ast, lhs, depth + 1, out);
            visualize_expr(&ast, rhs, depth + 1, out);
        }
        ASTExpr::Unary(op, expr) => {
            let op = &ast.tokens[op.0].slice;
            out.push_str(&format!("{indent}UnaryOp: {op:?}\n"));
            visualize_expr(&ast, expr, depth + 1, out);
        }
        ASTExpr::Apply(ref args) => {
            out.push_str(&format!("{indent}Apply:\n"));
            for &arg in args.iter() {
                visualize_expr(&ast, arg, depth + 1, out);
            }
        }
        ASTExpr::Empty => {
            out.push_str(&format!("{indent}Empty\n"));
        }
        ASTExpr::Path(root, ref next) => {
            out.push_str(&format!("{indent}Root: \n"));
            visualize_expr(&ast, root, depth + 1, out);
            out.push_str(&format!("{indent}Path: \n"));
            for &expr in next.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        ASTExpr::Braced(op, ref mid, cl) => {
            let op = &ast.tokens[op.0].slice;
            let cl = &ast.tokens[cl.0].slice;
            out.push_str(&format!("{indent}Braced: {op:?}, {cl:?}\n"));
            for &expr in mid.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        ASTExpr::Flow(lhs, op, mhs, ref rhs) => {
            let op = &ast[op].slice;
            out.push_str(&format!("{indent}Flow: {op:?}\n"));
            visualize_expr(&ast, lhs, depth + 1, out);
            if let Some(mhs) = mhs {
                out.push_str(&format!("{indent}Gets: \n"));
                visualize_expr(&ast, mhs, depth + 1, out);
            }
            out.push_str(&format!("{indent}And then: \n"));
            for &expr in rhs.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        ASTExpr::UnFlow(op, mhs, ref rhs) => {
            let op = &ast[op].slice;
            out.push_str(&format!("{indent}Flow: {op:?}\n"));
            if let Some(mhs) = mhs {
                out.push_str(&format!("{indent}Gets: \n"));
                visualize_expr(&ast, mhs, depth + 1, out);
            }
            out.push_str(&format!("{indent}And then: \n"));
            for &expr in rhs.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        ASTExpr::Tuple(ref m) => {
            out.push_str(&format!("{indent}Tuple: {}\n", m.len()));
            for &expr in m.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        ASTExpr::Either(ref m) => {
            out.push_str(&format!("{indent}Either: {}\n", m.len()));
            for &expr in m.iter() {
                visualize_expr(&ast, expr, depth + 1, out);
            }
        }
        #[allow(unreachable_patterns)]
        ref other => {
            out.push_str(&format!("{indent}{other:?} (not implemented)\n"));
        }
    }
}
