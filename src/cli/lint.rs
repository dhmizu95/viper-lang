use crate::semantic::type_checker::TypeChecker;

pub fn run_lint(args: &LintArgs) -> crate::error::Result<()> {
    let source = std::fs::read_to_string(&args.input).map_err(crate::error::ViperError::Io)?;

    let mut lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = crate::parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let mut type_checker = TypeChecker::new();
    let _ = type_checker.check(&ast);

    let mut warnings = Vec::new();

    // Check for unused variables (simple heuristic)
    let mut all_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_variables(&ast, &mut all_vars);

    let mut used_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    collect_used_variables(&ast, &mut used_vars);

    for var in all_vars.difference(&used_vars) {
        if !var.starts_with('_') {
            warnings.push(format!("Warning: unused variable '{}'", var));
        }
    }

    // Print results
    if warnings.is_empty() {
        println!("No issues found.");
    } else {
        for warning in &warnings {
            println!("{}", warning);
        }
        println!("\nFound {} issue(s).", warnings.len());
    }

    // Print type errors from type checker
    if !type_checker.errors().is_empty() {
        for error in type_checker.errors() {
            eprintln!("Error: {:?}", error);
        }
        return Err(crate::error::ViperError::cli(format!(
            "{} type error(s) found",
            type_checker.errors().len()
        )));
    }

    Ok(())
}

fn collect_variables(module: &crate::ast::Module, vars: &mut std::collections::HashSet<String>) {
    for stmt in &module.statements {
        collect_stmt_vars(stmt, vars);
    }
}

fn collect_stmt_vars(stmt: &crate::ast::Stmt, vars: &mut std::collections::HashSet<String>) {
    match stmt {
        crate::ast::Stmt::Declare { name, .. } => {
            vars.insert(name.clone());
        }
        crate::ast::Stmt::Assign { target, .. } => {
            if let crate::ast::Expr::Ident(n, _) = target.as_ref() {
                vars.insert(n.clone());
            }
        }
        crate::ast::Stmt::Function { body, .. } => {
            for s in body {
                collect_stmt_vars(s, vars);
            }
        }
        crate::ast::Stmt::If { body, elif_blocks, else_body, .. } => {
            for s in body {
                collect_stmt_vars(s, vars);
            }
            for (_, b) in elif_blocks {
                for s in b {
                    collect_stmt_vars(s, vars);
                }
            }
            if let Some(b) = else_body {
                for s in b {
                    collect_stmt_vars(s, vars);
                }
            }
        }
        crate::ast::Stmt::For { body, .. } | crate::ast::Stmt::While { body, .. } => {
            for s in body {
                collect_stmt_vars(s, vars);
            }
        }
        _ => {}
    }
}

fn collect_used_variables(
    module: &crate::ast::Module,
    vars: &mut std::collections::HashSet<String>,
) {
    for stmt in &module.statements {
        collect_stmt_usage(stmt, vars);
    }
}

fn collect_stmt_usage(stmt: &crate::ast::Stmt, vars: &mut std::collections::HashSet<String>) {
    use crate::ast::Stmt;

    match stmt {
        Stmt::Expr(expr) => {
            collect_expr_usage(expr, vars);
        }
        Stmt::Assign { value, .. } => {
            collect_expr_usage(value, vars);
        }
        Stmt::Declare { value, .. } => {
            if let Some(v) = value {
                collect_expr_usage(v, vars);
            }
        }
        Stmt::Function { body, .. } => {
            for s in body {
                collect_stmt_usage(s, vars);
            }
        }
        Stmt::If { body, elif_blocks, else_body, .. } => {
            for s in body {
                collect_stmt_usage(s, vars);
            }
            for (_, b) in elif_blocks {
                for s in b {
                    collect_stmt_usage(s, vars);
                }
            }
            if let Some(b) = else_body {
                for s in b {
                    collect_stmt_usage(s, vars);
                }
            }
        }
        Stmt::For { iter, body, .. } => {
            collect_expr_usage(iter, vars);
            for s in body {
                collect_stmt_usage(s, vars);
            }
        }
        Stmt::While { condition, body, .. } => {
            collect_expr_usage(condition, vars);
            for s in body {
                collect_stmt_usage(s, vars);
            }
        }
        _ => {}
    }
}

fn collect_expr_usage(expr: &crate::ast::Expr, vars: &mut std::collections::HashSet<String>) {
    use crate::ast::Expr;

    match expr {
        Expr::Ident(n, _) => {
            vars.insert(n.clone());
        }
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Str(_, _)
        | Expr::None(_) => {}
        Expr::BinOp { left, right, .. } => {
            collect_expr_usage(left, vars);
            collect_expr_usage(right, vars);
        }
        Expr::UnaryOp { operand, .. } => {
            collect_expr_usage(operand, vars);
        }
        Expr::Call { func, args, .. } => {
            collect_expr_usage(func, vars);
            for a in args {
                collect_expr_usage(a, vars);
            }
        }
        Expr::List { elements, .. } => {
            for e in elements {
                collect_expr_usage(e, vars);
            }
        }
        Expr::Index { obj, index, .. } => {
            collect_expr_usage(obj, vars);
            collect_expr_usage(index, vars);
        }
        Expr::Attribute { obj, .. } => {
            collect_expr_usage(obj, vars);
        }
        Expr::Lambda { body, .. } => {
            collect_expr_usage(body, vars);
        }
        Expr::Conditional { condition, then_expr, else_expr, .. } => {
            collect_expr_usage(condition, vars);
            collect_expr_usage(then_expr, vars);
            collect_expr_usage(else_expr, vars);
        }
        Expr::Await { future, .. } => {
            collect_expr_usage(future, vars);
        }
        _ => {}
    }
}

pub struct LintArgs {
    pub input: String,
    pub warnings: bool,
}

impl LintArgs {
    pub fn new(input: String, warnings: bool) -> Self {
        Self { input, warnings }
    }
}
