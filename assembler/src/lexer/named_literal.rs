use phf::phf_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedLiteral {
    SP,
    X,
    A,
    Y,
    CC,
    XPlus,
    XMinus,
    PlusX,
    MinusX,
    YPlus,
    YMinus,
    PlusY,
    MinusY,
}

static NAMED_LITERAL: phf::Map<&'static str, NamedLiteral> = phf_map! {
    "SP" => NamedLiteral::SP,
    "X" => NamedLiteral::X,
    "A" => NamedLiteral::A,
    "Y" => NamedLiteral::Y,
    "CC" => NamedLiteral::CC,
    "X+" => NamedLiteral::XPlus,
    "X-" => NamedLiteral::XMinus,
    "+X" => NamedLiteral::PlusX,
    "-X" => NamedLiteral::MinusX,
    "Y+" => NamedLiteral::YPlus,
    "Y-" => NamedLiteral::YMinus,
    "+Y" => NamedLiteral::PlusY,
    "-Y" => NamedLiteral::MinusY,
};

pub fn parse_named_literal(s: &str) -> Option<NamedLiteral> {
    NAMED_LITERAL.get(s).copied()
}
