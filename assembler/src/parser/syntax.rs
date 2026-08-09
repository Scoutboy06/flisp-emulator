use std::ops::Range;

use crate::lexer::NamedLiteral;

#[derive(Debug, Clone)]
pub enum Atom {
    Expr(Expression),
    Reg(NamedLiteral),
    String(String),
    None,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number { value: u8, span: Range<usize> },
    Symbol { name: String, span: Range<usize> },
}

impl Expression {
    pub fn span(&self) -> &Range<usize> {
        match self {
            Self::Number { span, .. } | Self::Symbol { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum OperandForm {
    None,

    /// Just one operand: `n`, `X`, label, etc.
    One(Atom),

    /// Something like `n,X` or `label,Y`
    Two(Atom, Atom),

    /// Immediate: `#5` or `#label`
    Imm1(Atom),
}
