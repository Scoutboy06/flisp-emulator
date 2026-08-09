#[allow(clippy::module_inception)]
mod instruction_selection;
mod parser;
mod syntax;

pub use parser::*;

pub use instruction_selection::Operand;
pub use syntax::{Atom, Expression};
