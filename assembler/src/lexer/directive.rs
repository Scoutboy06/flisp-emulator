use phf::phf_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directive {
    ORG,
    EQU,
    FCB,
    FCS,
    RMB,
}

static DIRECTIVE: phf::Map<&'static str, Directive> = phf_map! {
    "ORG" => Directive::ORG,
    "EQU" => Directive::EQU,
    "FCB" => Directive::FCB,
    "FCS" => Directive::FCS,
    "RMB" => Directive::RMB,
};

pub fn parse_directive(s: &str) -> Option<Directive> {
    DIRECTIVE.get(s).copied()
}
