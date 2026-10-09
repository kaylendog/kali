//! Parser tests. One module per grammar nonterminal; a test lives in the module of the
//! outermost construct its input exercises. Tests assert acceptance, rejection, or AST shape
//! where the printed form cannot show it. Exact output belongs in kali-print's tests.

mod common;

mod exprs;
mod items;
mod lexer;
mod patterns;
mod spec;
mod types;
