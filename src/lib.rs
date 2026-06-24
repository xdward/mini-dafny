//! Verification system for testing programs.
//!
//! A program can be broken down into components that are tested individually for logical
//! correctness. By specifying each component’s expectations and constraints, formulas can
//! be derived and verified with theorem solvems such as
//! [Z3](https://www.microsoft.com/en-us/research/project/z3/). This library provides a small
//! imperative language for writing verifiable specifications.
//!
//! # Writing Specifications
//!
//! A specification can be written with simple psuedocode. Its basic building blocks are values,
//! expressions, and statements.
//!
//! ### Expressions and Values
//!
//! ##### Values
//!
//! * Integers (64-bit signed)
//! * Booleans (`TRUE` or `FALSE`)
//! * Variables holding integers
//!
//! ##### Arithmetic Expressions
//!
//! * Expressions that are evaluated into an integer value or polynomial.
//! * Includes integers, variables, and arithmetic [operators](lexer::Symbol).
//! * **Division operations are floored**.
//!
//! ```text
//! 21
//! 11 / (5 + 3)
//! ```
//!
//! ##### Boolean Expressions
//!
//! * Expressions that are evaluated into a boolean value.
//! * Includes booleans and expressions with relational/logical [connectors](lexer::Symbol).
//!
//! ```text
//! FALSE
//! y > 10 || x == 0 && TRUE
//! ```
//!
//! *Expressions are evaluated left to right.*
//!
//! ### Statements
//!
//! ##### Variable Declaration
//!
//! 1. Only one declaration is allowed; if variables exist, declare them in the first line.
//! 2. Variables names are alphanumeric (underscores allowed); the first character must be a letter.
//! 3. Multiple variables may be declared, separated by commas.
//!
//! ```text
//! var radius, xPoint, yPoint, LINE_WIDTH
//! ```
//!
//! ##### Assumption
//!
//! 1. Must be defined after the variable declaration; multiple assumptions are allowed.
//! 2. Must be a boolean expression.
//!
//! ```text
//! assume x == 0 && y == 0
//! assume x == y
//! ```
//!
//! ##### Assertion
//!
//! 1. Must be defined at the end of a specification; multiple assertions are allowed.
//! 2. Must be a boolean expression.
//!
//! ```text
//! assert x > 0
//! assert x != 0
//! ```
//!
//! ##### Assignment
//!
//! 1. The right-hand side must be an arithmetic expression or integer value.
//! 2. **Assigning a boolean expression or value to a variable is not supported**.
//!
//! ```text
//! x := x + 1
//! ```
//!
//! ##### If Conditional
//!
//! 1. The condition must be a boolean expression.
//! 2. The else branch is optional, but must be specified with the `else` keyword if included.
//! 3. The `end` keyword is required to close the conditional.
//!
//! ```text
//! if x > y then
//!    x += 1
//! else
//!    y += 1
//! end
//! ```
//!
//! ##### While Loop
//!
//! 1. The condition and invariant must be boolean expressions.
//! 2. Providing a [loop invariant](https://en.wikipedia.org/wiki/Loop_invariant) is required.
//! 3. The `end` keyword is required to close the loop.
//!
//! ```text
//! while x > 0
//! invariant x >=0
//!     x := x - 1
//! end
//! ```
//!
//! *Indentation is not enforced.*
//!
//! # Usage
//!
//! Call the verification function with the specification as an argument. It will,
//!
//! 1. Tokenize the specification code.
//! 2. Parse an intermediate representation.
//! 3. Generate a verification condition.
//! 4. Evaluate the formula with [Z3](https://www.microsoft.com/en-us/research/project/z3/).
//!
//! The output can be read from the `message` field of the returned [Response].
//!
//! ```rust,no_run
//! # use verifier::verify;
//! # let spec = "assert TRUE";
//! let result = verify(spec);
//! println!("{}", result.message);
//! ```

use wasm_bindgen::prelude::*;
use z3::Solver;

pub mod ast;
pub mod compiler;
pub mod errors;
pub mod expr;
pub mod lexer;
pub mod parser;

/// Result produced by the verifier.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierResult {
    /// The specification is provably correct.
    Correct,
    /// A counterexample has been found, so the specification is incorrect.
    Counterexample,
    /// An error occured before or during verification.
    Err,
}

/// Contains the verification result and message.
#[wasm_bindgen(getter_with_clone)]
pub struct Response {
    /// The verifier's assessment of the specification.
    pub result: VerifierResult,
    /// Contains error data, a counterexample, or confirmation of success.
    pub message: String,
}

/// Verifies the correctness of a specification.
#[wasm_bindgen]
pub fn verify(input: &str) -> Response {
    let tokens = match lexer::tokenize(input) {
        Ok(result) => result,
        Err(err) => {
            return Response {
                result: VerifierResult::Err,
                message: format!("error[LEXER]: {}", err),
            };
        }
    };
    let mut ast = match parser::parse(&tokens) {
        Ok(result) => result,
        Err(err) => {
            return Response {
                result: VerifierResult::Err,
                message: format!("error[PARSER]: {}", err),
            };
        }
    };
    let (vc, env) = match compiler::compile(&mut ast) {
        Ok(result) => result,
        Err(err) => {
            return Response {
                result: VerifierResult::Err,
                message: format!("error[COMPILER]: {}", err),
            };
        }
    };

    // The negation of the verification condition is used to prove the correctness of the
    // specification. That means, a result of "sat" would provide a counterexample model.
    let solver = Solver::new();
    solver.assert(vc.not());

    match solver.check() {
        z3::SatResult::Unsat => Response {
            result: VerifierResult::Correct,
            message: "specification is correct!".to_string(),
        },
        z3::SatResult::Unknown => Response {
            result: VerifierResult::Counterexample,
            message: "err[Z3]: unexpected error".to_string(),
        },
        z3::SatResult::Sat => {
            let mut msg = String::from("counterexample:\n\n");
            let model = solver.get_model().unwrap();
            for (id, z3_int_const) in env.iter() {
                let solution = model.eval(z3_int_const, false).unwrap();
                // ignore variables used in the counterexample
                if !(solution == z3_int_const) {
                    msg += &format!("{} = {}\n", id, solution)
                }
            }

            Response {
                result: VerifierResult::Counterexample,
                message: msg,
            }
        }
    }
}
