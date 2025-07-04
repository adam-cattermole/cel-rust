#[allow(clippy::all)]
mod gen;

pub mod ast;
pub mod reference;
pub mod references;

#[cfg(feature = "protobuf")]
pub mod protobuf;

pub use ast::IdedExpr as Expression;

mod macros;
mod parse;
#[allow(non_snake_case)]
mod parser;

pub use parser::*;
pub use references::ExpressionReferences;
