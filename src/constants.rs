

pub const PRINT_SUCCESS: &str = "print-success";
pub const PRODUCE_MODELS: &str = "produce-models";
pub const AND: &str = "and";
pub const OR: &str = "or";
pub const NOT: &str = "not";
pub const IMPLICATION: &str = "=>";
pub const XOR: &str = "xor";
pub const EQUALITY: &str = "=";

pub enum Logic {
    QfUf,
    QfLia
}


impl Logic {
    pub fn new(logic: &str) -> Self {
        match logic {
            "QF_UF" => Self::QfUf,
            "QF_LIA" => Self::QfLia,
            "QF_LRA" => unimplemented!(),
            _ => panic!("Invalid logic")
        }
    }
}