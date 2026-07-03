//! Transpiler for creating Z3 formulas.
use std::collections::HashMap;

use z3::ast::{self, Ast};

use crate::{
    ast::{Block, Expr, Stmt},
    errors::CompileError,
    lexer::Symbol,
};

/// Alias for Z3 [boolean nodes](z3::ast::Bool).
type Z3Bool = ast::Bool;
/// Alias for Z3 [integer nodes](z3::ast::Int).
type Z3Int = ast::Int;
/// Alias for the environment.
///
/// Implemented as a [HashMap], where the key is the variable name (i.e. `"x"`) and the value is an
/// owned [Int](z3::ast::Int) constant from Z3.
type Env = HashMap<String, Z3Int>;

/// Represents the SMT encoding of boolean/arithmetic expressions.
#[derive(Debug, PartialEq, Eq)]
enum Encoding {
    Bool(Z3Bool),
    Int(Z3Int),
}

impl Encoding {
    /// Evaluates the encoding of a boolean expression.
    fn eval_bool(&self) -> Result<Z3Bool, CompileError> {
        match self {
            Self::Bool(z3_bool) => Ok(z3_bool.to_owned()),
            _ => Err(CompileError::BadExpression),
        }
    }

    // Evaluates the encoding of an arithmetic expression.
    fn eval_int(&self) -> Result<Z3Int, CompileError> {
        match self {
            Self::Int(z3_int) => Ok(z3_int.to_owned()),
            _ => Err(CompileError::BadExpression),
        }
    }
}

impl Expr {
    /// Encodes an expression into a model that can be evaluated.
    fn smt_encode(&self, env: &Env) -> Result<Encoding, CompileError> {
        match self {
            Expr::Boolean(b) => Ok(Encoding::Bool(Z3Bool::from_bool(*b))),
            Expr::Integer(i) => Ok(Encoding::Int(Z3Int::from_i64(*i))),
            Expr::Variable(x) => match env.get(x) {
                Some(z3_int) => Ok(Encoding::Int(z3_int.clone())),
                _ => Err(CompileError::UnboundVariable),
            },
            Expr::BinOp { lhs, op, rhs } => {
                match (lhs.smt_encode(env)?, op, rhs.smt_encode(env)?) {
                    (Encoding::Int(a), Symbol::Add, Encoding::Int(b)) => Ok(Encoding::Int(a + b)),
                    (Encoding::Int(a), Symbol::Sub, Encoding::Int(b)) => Ok(Encoding::Int(a - b)),
                    (Encoding::Int(a), Symbol::Mul, Encoding::Int(b)) => Ok(Encoding::Int(a * b)),
                    (Encoding::Int(a), Symbol::Div, Encoding::Int(b)) => Ok(Encoding::Int(a / b)),
                    (Encoding::Int(a), Symbol::Lt, Encoding::Int(b)) => Ok(Encoding::Bool(a.lt(b))),
                    (Encoding::Int(a), Symbol::Gt, Encoding::Int(b)) => Ok(Encoding::Bool(a.gt(b))),
                    (Encoding::Int(a), Symbol::Le, Encoding::Int(b)) => Ok(Encoding::Bool(a.le(b))),
                    (Encoding::Int(a), Symbol::Ge, Encoding::Int(b)) => Ok(Encoding::Bool(a.ge(b))),
                    (Encoding::Int(a), Symbol::Eq, Encoding::Int(b)) => Ok(Encoding::Bool(a.eq(b))),
                    (Encoding::Int(a), Symbol::Neq, Encoding::Int(b)) => {
                        Ok(Encoding::Bool(a.ne(b)))
                    }
                    (Encoding::Bool(p), Symbol::Eq, Encoding::Bool(q)) => {
                        Ok(Encoding::Bool(p.eq(q)))
                    }
                    (Encoding::Bool(p), Symbol::Neq, Encoding::Bool(q)) => {
                        Ok(Encoding::Bool(p.ne(q)))
                    }
                    (Encoding::Bool(p), Symbol::And, Encoding::Bool(q)) => {
                        Ok(Encoding::Bool(Z3Bool::and(&[&p, &q])))
                    }
                    (Encoding::Bool(p), Symbol::Or, Encoding::Bool(q)) => {
                        Ok(Encoding::Bool(Z3Bool::or(&[&p, &q])))
                    }
                    _ => Err(CompileError::BadExpression),
                }
            }
        }
    }
}

impl Stmt {
    /// Calculates the weakest precondition of a statement.
    ///
    /// See: <https://en.wikipedia.org/wiki/Predicate_transformer_semantics#Weakest_preconditions>
    fn wp(&self, env: &Env, post: &Z3Bool) -> Result<Z3Bool, CompileError> {
        match self {
            Stmt::Declare(_) => Err(CompileError::InvalidDeclaration),
            Stmt::Assume(expr) => Ok(expr.smt_encode(env)?.eval_bool()?.implies(post)),
            Stmt::Assert(expr) => {
                let cond = expr.smt_encode(env)?.eval_bool()?;
                let p = cond.implies(post);
                let q = cond.not().implies(Z3Bool::from_bool(false));
                Ok(Z3Bool::and(&[&p, &q]))
            }
            Stmt::Assign(id, expr) => match env.get(id) {
                Some(z3_int) => {
                    Ok(post.substitute(&[(z3_int, &expr.smt_encode(env)?.eval_int()?)]))
                }
                None => Err(CompileError::UnboundVariable),
            },
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = condition.smt_encode(env)?.eval_bool()?;
                let then_wp = then_branch.wp(env, post)?;
                let else_wp = match else_branch {
                    Some(block) => block.wp(env, post)?,
                    None => post.to_owned(),
                };

                Ok(Z3Bool::and(&[
                    &cond.implies(then_wp),
                    &cond.not().implies(else_wp),
                ]))
            }
            Stmt::While {
                condition,
                invariant,
                body,
            } => {
                let cond = condition.smt_encode(env)?.eval_bool()?;
                let inv = invariant.smt_encode(env)?.eval_bool()?;
                let loop_wp = body.wp(env, &inv)?;

                let bounds: Vec<&dyn Ast> = env.values().map(|v| v as &dyn Ast).collect();

                Ok(Z3Bool::and(&[
                    &inv,
                    &ast::forall_const(
                        &bounds,
                        &[],
                        &Z3Bool::and(&[
                            &Z3Bool::and(&[&cond, &inv]).implies(loop_wp),
                            &Z3Bool::and(&[&cond.not(), &inv]).implies(post),
                        ]),
                    ),
                ]))
            }
            _ => Err(CompileError::BadStatement),
        }
    }
}

impl Block {
    /// Calculates the weakest precondition for a sequence of statements.
    fn wp(&self, env: &Env, post: &Z3Bool) -> Result<Z3Bool, CompileError> {
        let mut expr = post.to_owned();
        for stmt in self.0.iter().rev() {
            expr = stmt.wp(env, &expr)?;
        }
        Ok(expr)
    }
}

/// Generates a verification condition from a specification's AST.
pub fn compile(ast: &mut Block) -> Result<(Z3Bool, Env), CompileError> {
    let mut env = HashMap::new();

    if let Stmt::Declare(ref identifiers) = ast.0[0] {
        for x in identifiers {
            if env.contains_key(x) {
                return Err(CompileError::AlreadyDeclared);
            }
            env.insert(x.clone(), Z3Int::fresh_const(x));
        }

        // Removes the only allowed declaration; any additional declarations will cause an error.
        // There is a minor O(n) performance cost because subsequent statements must be shiftef
        // left. Since this happens only once, it should be acceptable. Alternatives such as using
        // [VecDeque] mitigate this cost, but add overhead to parsing and AST construction.
        ast.0.remove(0);
    }

    // pre and post condition set to true
    let pre = Z3Bool::from_bool(true);
    let post = Z3Bool::from_bool(true);

    Ok((pre.implies(ast.wp(&mut env, &post)?), env))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_encoding() {
        assert_eq!(
            Expr::Boolean(true).smt_encode(&mut HashMap::new()).unwrap(),
            Encoding::Bool(Z3Bool::from_bool(true))
        );
    }

    #[test]
    fn test_bool_encoding() {
        assert_eq!(
            Expr::Integer(2).smt_encode(&mut HashMap::new()).unwrap(),
            Encoding::Int(Z3Int::from_i64(2))
        );
    }

    #[test]
    fn test_variable_encoding() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Z3Int::fresh_const("x"));

        assert_eq!(
            Expr::Variable("x".to_string())
                .smt_encode(&mut env)
                .unwrap(),
            Encoding::Int(env.get("x").unwrap().to_owned())
        );
    }

    #[test]
    fn test_arithmetic_eval() {
        assert_eq!(
            Expr::BinOp {
                lhs: Box::new(Expr::BinOp {
                    lhs: Box::new(Expr::Integer(4)),
                    op: Symbol::Sub,
                    rhs: Box::new(Expr::BinOp {
                        lhs: Box::new(Expr::Integer(7)),
                        op: Symbol::Mul,
                        rhs: Box::new(Expr::Integer(-1))
                    })
                }),
                op: Symbol::Add,
                rhs: Box::new(Expr::BinOp {
                    // note: 3 // 2 = 1
                    lhs: Box::new(Expr::Integer(3)),
                    op: Symbol::Div,
                    rhs: Box::new(Expr::Integer(2))
                })
            }
            .smt_encode(&mut HashMap::new())
            .unwrap()
            .eval_int()
            .unwrap(),
            Z3Int::from_i64(4) - (Z3Int::from_i64(7) * Z3Int::from_i64(-1))
                + (Z3Int::from_i64(3) / Z3Int::from_i64(2))
        );
    }

    #[test]
    fn test_boolean_eval() {
        assert_eq!(
            Expr::BinOp {
                lhs: Box::new(Expr::BinOp {
                    lhs: Box::new(Expr::Boolean(true)),
                    op: Symbol::Eq,
                    rhs: Box::new(Expr::BinOp {
                        lhs: Box::new(Expr::Integer(0)),
                        op: Symbol::Gt,
                        rhs: Box::new(Expr::Integer(1))
                    })
                }),
                op: Symbol::Or,
                rhs: Box::new(Expr::Boolean(true))
            }
            .smt_encode(&mut HashMap::new())
            .unwrap()
            .eval_bool()
            .unwrap(),
            Z3Bool::or(&[
                &Z3Bool::from_bool(true).eq(Z3Int::from_i64(0).gt(Z3Int::from_i64(1))),
                &Z3Bool::from_bool(true)
            ])
        );
    }

    #[test]
    fn test_polynomial_expr() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Z3Int::fresh_const("x"));

        assert_eq!(
            Expr::BinOp {
                lhs: Box::new(Expr::Variable("x".to_string())),
                op: Symbol::Add,
                rhs: Box::new(Expr::Integer(1))
            }
            .smt_encode(&mut env)
            .unwrap()
            .eval_int()
            .unwrap(),
            env.get("x").unwrap() + Z3Int::from_i64(1)
        );
    }

    #[test]
    fn test_assume_wp() {
        assert_eq!(
            Stmt::Assume(Expr::Boolean(false))
                .wp(&HashMap::new(), &Z3Bool::from_bool(true))
                .unwrap(),
            Z3Bool::from_bool(false).implies(Z3Bool::from_bool(true))
        );
    }

    #[test]
    fn test_assert_wp() {
        assert_eq!(
            Stmt::Assert(Expr::Boolean(false))
                .wp(&HashMap::new(), &Z3Bool::from_bool(true))
                .unwrap(),
            Z3Bool::and(&[
                &Z3Bool::from_bool(false).implies(Z3Bool::from_bool(true)),
                &Z3Bool::from_bool(false)
                    .not()
                    .implies(Z3Bool::from_bool(false))
            ])
        );
    }

    #[test]
    fn test_assign_wp() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Z3Int::fresh_const("x"));

        assert_eq!(
            Stmt::Assign("x".to_string(), Expr::Integer(1))
                .wp(&env, &env["x"].ne(Z3Int::from_i64(0)))
                .unwrap(),
            Z3Int::from_i64(1).ne(Z3Int::from_i64(0))
        );
    }

    #[test]
    fn test_if_wp() {
        assert_eq!(
            Stmt::If {
                condition: Expr::Boolean(false),
                then_branch: Block(vec![]),
                else_branch: None,
            }
            .wp(&HashMap::new(), &Z3Bool::from_bool(true))
            .unwrap(),
            Z3Bool::and(&[
                &Z3Bool::from_bool(false).implies(Z3Bool::from_bool(true)),
                &Z3Bool::from_bool(false)
                    .not()
                    .implies(Z3Bool::from_bool(true))
            ])
        );
    }

    #[test]
    fn test_while_wp() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Z3Int::fresh_const("x"));

        assert_eq!(
            Stmt::While {
                condition: Expr::Boolean(false),
                invariant: Expr::Boolean(false),
                body: Block(vec![])
            }
            .wp(&env, &Z3Bool::from_bool(true))
            .unwrap(),
            Z3Bool::and(&[
                &Z3Bool::from_bool(false),
                &ast::forall_const(
                    &[&env["x"]],
                    &[],
                    &Z3Bool::and(&[
                        &Z3Bool::and(&[&Z3Bool::from_bool(false), &Z3Bool::from_bool(false)])
                            .implies(Z3Bool::from_bool(false)), // loop postcondition is invariant
                        &Z3Bool::and(&[&Z3Bool::from_bool(false).not(), &Z3Bool::from_bool(false)])
                            .implies(Z3Bool::from_bool(true))
                    ],)
                )
            ])
        );
    }

    #[test]
    fn test_block() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Z3Int::fresh_const("x"));

        assert_eq!(
            Block(vec![
                Stmt::Assign("x".to_string(), Expr::Integer(40)),
                Stmt::Assign(
                    "x".to_string(),
                    Expr::BinOp {
                        lhs: Box::new(Expr::Variable("x".to_string())),
                        op: Symbol::Div,
                        rhs: Box::new(Expr::Integer(5)),
                    },
                ),
                Stmt::Assert(Expr::BinOp {
                    lhs: Box::new(Expr::Variable("x".to_string())),
                    op: Symbol::Eq,
                    rhs: Box::new(Expr::Integer(8)),
                }),
            ])
            .wp(&env, &Z3Bool::from_bool(true))
            .unwrap(),
            Z3Bool::and(&[
                &(Z3Int::from_i64(40) / Z3Int::from_i64(5))
                    .eq(Z3Int::from_i64(8))
                    .implies(Z3Bool::from_bool(true)),
                &(Z3Int::from_i64(40) / Z3Int::from_i64(5))
                    .eq(Z3Int::from_i64(8))
                    .not()
                    .implies(Z3Bool::from_bool(false))
            ])
        );
    }

    #[test]
    fn test_err_invalid_declare() {
        assert_eq!(
            compile(&mut Block(vec![
                Stmt::Declare(vec!["x".to_string()]),
                Stmt::Declare(vec!["x".to_string()]),
            ]))
            .unwrap_err(),
            CompileError::InvalidDeclaration
        );
    }

    #[test]
    fn test_err_repeated_variable() {
        assert_eq!(
            compile(&mut Block(vec![Stmt::Declare(vec![
                "x".to_string(),
                "x".to_string()
            ]),]))
            .unwrap_err(),
            CompileError::AlreadyDeclared
        );
    }

    #[test]
    fn test_err_unbound_variable() {
        assert_eq!(
            compile(&mut Block(vec![
                Stmt::Declare(vec!["x".to_string()]),
                Stmt::Assign("y".to_string(), Expr::Integer(1)),
            ]))
            .unwrap_err(),
            CompileError::UnboundVariable
        );
    }
}
