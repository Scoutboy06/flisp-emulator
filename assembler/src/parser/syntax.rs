use crate::lexer::NamedLiteral;

#[derive(Debug, Clone)]
pub enum Atom {
    NumOrSym(NumOrSym),
    Reg(NamedLiteral),
    String(String),
    None,
}

#[derive(Debug, Clone)]
pub enum NumOrSym {
    Num(u8),
    Sym(String),
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
