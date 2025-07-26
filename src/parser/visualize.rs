use crate::parser::{AST, ASTExpr, ASTErr, ExprInd, };

pub fn visualize_ast(ast: &AST) -> String {
    let mut output = String::new();

    for (i, (token_ind, expr)) in ast.expressions.iter().enumerate() {
        let expr_ind = ExprInd(i);
        let indent = "  ";

        // Заголовок
        output += &format!("[Expr #{i}] @ {:?}:\n", token_ind);

        // Тело выражения
        visualize_expr(ast, expr, 1, &mut output);

        // Ошибки (если есть)
        for (err_ind, err) in &ast.errors {
            if *err_ind == expr_ind {
                output += &format!("{indent}⚠️  Error: {}\n", format_ast_error(err));
            }
        }

        output += "\n";
    }

    output
}

fn visualize_expr(ast: &AST, expr: &ASTExpr, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    match expr {
        ASTExpr::Simple(lit) => {
            let lit = &ast.tokens[lit.0].slice;
            out.push_str(&format!("{indent}Simple: {lit:?}\n"));
        }
        ASTExpr::Binary(lhs, op, rhs) => {
            let op = &ast.tokens[op.0].slice;
            out.push_str(&format!("{indent}Binary: {op:?}\n"));
            visualize_expr(&ast, &ast.expressions[lhs.0].1, depth + 1, out);
            visualize_expr(&ast, &ast.expressions[rhs.0].1, depth + 1, out);
        }
        ASTExpr::Unary(op, expr) => {
            let op = &ast.tokens[op.0].slice;
            out.push_str(&format!("{indent}UnaryOp: {op:?}\n"));
            visualize_expr(&ast, &ast.expressions[expr.0].1, depth + 1, out);
        }
        ASTExpr::Apply(func, args) => {
            out.push_str(&format!("{indent}Apply:\n"));
            visualize_expr(&ast, &ast.expressions[func.0].1, depth + 1, out);
            out.push_str(&format!("{indent} Args:\n"));
            for arg in args {
                visualize_expr(&ast, &ast.expressions[arg.0].1, depth + 1, out);
            }
        }
        ASTExpr::Empty => {
            out.push_str(&format!("{indent}Empty\n"));
        }
        ASTExpr::Path(root, next) => {
            out.push_str(&format!("{indent}Root: \n"));
            visualize_expr(&ast, &ast.expressions[root.0].1, depth + 1, out);
            out.push_str(&format!("{indent} Path: \n"));
            for expr in next {
                visualize_expr(&ast, &ast.expressions[expr.0].1, depth + 1, out);
            }            
        }
        ASTExpr::Braced(op, mid, cl) => {
            let op = &ast.tokens[op.0].slice;
            let cl = &ast.tokens[cl.0].slice;
            out.push_str(&format!("{indent}Braced: {op:?}, {cl:?}\n"));
            for expr in mid {
                visualize_expr(&ast, &ast.expressions[expr.0].1, depth + 1, out);
            }
        }
        #[allow(unreachable_patterns)]
        other => {
            out.push_str(&format!("{indent}{other:?} (not implemented)\n"));
        }
    }
}

fn format_ast_error(err: &ASTErr) -> String {
    match err {
        ASTErr::Unopened => "Unopened".to_string(),
        ASTErr::Unclosed => "Unclosed".to_string(),
        ASTErr::NotAtomic => "Not Atomic".to_string(),
    }
}