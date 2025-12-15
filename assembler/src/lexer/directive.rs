use phf::phf_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directive {
    Org,
    Equ,
    Fcb,
    Fcs,
    Rmb,
}

static DIRECTIVE: phf::Map<&'static str, Directive> = phf_map! {
    "ORG" => Directive::Org,
    "EQU" => Directive::Equ,
    "FCB" => Directive::Fcb,
    "FCS" => Directive::Fcs,
    "RMB" => Directive::Rmb,
};

pub fn parse_directive(s: &str) -> Option<Directive> {
    DIRECTIVE.get(s).copied()
}
