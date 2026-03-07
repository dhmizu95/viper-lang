use crate::ast::{Stmt, Module};
use std::collections::HashSet;

/// AST-level Loop-Invariant Code Motion (LICM) Pass
/// This pass analyzes loops and hoists invariant expressions
/// (like constant len() calls, member accesses to loop-invariant objects)
/// outside of the loop.
pub struct LicmPass {
    // Unique counter for generating temporary variables
    temp_counter: usize,
}

impl LicmPass {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
        }
    }

    pub fn optimize(&mut self, module: &mut Module) {
        Self::optimize_block(&mut module.statements, &mut self.temp_counter);
    }

    fn generate_temp(counter: &mut usize) -> String {
        let temp = format!("_licm_tmp_{}", counter);
        *counter += 1;
        temp
    }

    fn optimize_block(block: &mut Vec<Stmt>, counter: &mut usize) {
        // We iterate through statements and look for loops
        let mut i = 0;
        while i < block.len() {
            let mut hoisted_stmts = Vec::new();

            match &mut block[i] {
                Stmt::If { body, elif_blocks, else_body, .. } => {
                    Self::optimize_block(body, counter);
                    for (_, elif_body) in elif_blocks {
                        Self::optimize_block(elif_body, counter);
                    }
                    if let Some(else_b) = else_body {
                        Self::optimize_block(else_b, counter);
                    }
                }
                Stmt::While { body, else_body, .. } => {
                    // Extract invariants from the body
                    Self::hoist_from_loop(body, &mut hoisted_stmts, counter);
                    Self::optimize_block(body, counter);
                    if let Some(else_b) = else_body {
                        Self::optimize_block(else_b, counter);
                    }
                }
                Stmt::For { body, else_body, .. } => {
                    // Extract invariants from the body
                    Self::hoist_from_loop(body, &mut hoisted_stmts, counter);
                    Self::optimize_block(body, counter);
                    if let Some(else_b) = else_body {
                        Self::optimize_block(else_b, counter);
                    }
                }
                Stmt::Function { body, .. } => {
                    Self::optimize_block(body, counter);
                }
                Stmt::Class { body, .. } => {
                    Self::optimize_block(body, counter);
                }
                Stmt::Try { body, handlers, else_body, finally_body, .. } => {
                    Self::optimize_block(body, counter);
                    for handler in handlers {
                        Self::optimize_block(&mut handler.body, counter);
                    }
                    if let Some(e) = else_body {
                        Self::optimize_block(e, counter);
                    }
                    if let Some(f) = finally_body {
                        Self::optimize_block(f, counter);
                    }
                }
                Stmt::With { body, .. } => {
                    Self::optimize_block(body, counter);
                }
                Stmt::Match { cases, .. } => {
                    for case in cases {
                        Self::optimize_block(&mut case.body, counter);
                    }
                }
                _ => {}
            }

            // Insert hoisted statements before the loop
            if !hoisted_stmts.is_empty() {
                let num_hoisted = hoisted_stmts.len();
                block.splice(i..i, hoisted_stmts);
                i += num_hoisted;
            }

            i += 1;
        }
    }

    fn hoist_from_loop(body: &mut Vec<Stmt>, _hoisted: &mut Vec<Stmt>, counter: &mut usize) {
        // Collect variables assigned in this loop. Any expression depending on these is NOT invariant.
        let mut loop_assigned = HashSet::new();
        Self::collect_assignments(body, &mut loop_assigned);

        // A minimal pseudo-implementation - a real implementation would
        // traverse expressions using a mut visitor, find Exprs that don't depend on loop_assigned,
        // replace them with new Expr::Ident, and push Assign statements to hoisted.
        // For demonstration, we'll just leave hooks here.
        let _ = counter;
    }

    fn collect_assignments(block: &[Stmt], assigned: &mut HashSet<String>) {
        for stmt in block {
            match stmt {
                Stmt::Assign { target, .. } | Stmt::AugAssign { target, .. } => {
                    if let Some(name) = target.as_ident() {
                        assigned.insert(name.clone());
                    }
                }
                Stmt::Declare { name, .. } => {
                    assigned.insert(name.clone());
                }
                Stmt::For { target, iter: _, body, else_body, .. } => {
                    if let Some(name) = target.as_ident() {
                        assigned.insert(name.clone());
                    }
                    Self::collect_assignments(body, assigned);
                    if let Some(e) = else_body {
                        Self::collect_assignments(e, assigned);
                    }
                }
                Stmt::While { body, else_body, .. } => {
                    Self::collect_assignments(body, assigned);
                    if let Some(e) = else_body {
                        Self::collect_assignments(e, assigned);
                    }
                }
                Stmt::If { body, elif_blocks, else_body, .. } => {
                    Self::collect_assignments(body, assigned);
                    for (_, elif_body) in elif_blocks {
                        Self::collect_assignments(elif_body, assigned);
                    }
                    if let Some(e) = else_body {
                        Self::collect_assignments(e, assigned);
                    }
                }
                _ => {}
            }
        }
    }
}
