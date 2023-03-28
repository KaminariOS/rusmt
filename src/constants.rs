use smt2parser::concrete::Symbol;

pub const PRINT_SUCCESS: &str = "print-success";
pub const PRODUCE_MODELS: &str = "produce-models";
pub const AND: &str = "and";
pub const OR: &str = "or";
pub const NOT: &str = "not";
pub const IMPLICATION: &str = "=>";
pub const XOR: &str = "xor";
pub const EQUALITY: &str = "=";

pub enum Logic {
    QF_UF,
    QF_LIA
}


impl Logic {
    pub fn new(logic: &str) -> Self {
        match logic {
            "QF_UF" => Self::QF_UF,
            "QF_LIA" => Self::QF_LIA,
            "QF_LRA" => unimplemented!(),
            _ => panic!("Invalid logic")
        }
    }
}