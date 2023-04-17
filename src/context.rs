use crate::assertion_set::{and, equality, implication, or, xor, AssertionSet, Clause, Literal};
use crate::constants::*;
use crate::get_id;
use smt2parser::concrete::{
    AttributeValue, Command, Identifier, Keyword, QualIdentifier, Symbol, Term,
};
use smt2parser::Numeral;

pub struct Context {
    logic: Option<Logic>,
    print_success: bool,
    produce_models: bool,
    exit: bool,
    assertion_sets: Vec<AssertionSet>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            logic: None,
            print_success: false,
            produce_models: false,
            exit: false,
            assertion_sets: vec![AssertionSet::default()],
        }
    }
}

impl Context {
    pub fn process_commands(&mut self, commands: Vec<Command>) {
        for command in commands {
            if self.exit {
                break;
            }
            self.process_command(command);
        }
    }

    pub fn no_logic(&self) -> bool {
        self.logic.is_none()
    }

    pub fn solve(&mut self) {}

    fn parse_term(&mut self, term: Term, clauses: &mut Vec<Clause>) -> Literal {
        match term {
            Term::QualIdentifier(QualIdentifier::Simple {
                identifier: Identifier::Simple { symbol },
            }) => Literal::new(self.get_symbol_id(symbol)),
            Term::Application {
                qual_identifier,
                arguments,
            } => match qual_identifier {
                QualIdentifier::Simple {
                    identifier:
                        Identifier::Simple {
                            symbol: Symbol(symbol),
                        },
                } => {
                    let args: Vec<Literal> = arguments
                        .into_iter()
                        .map(|term| self.parse_term(term, clauses))
                        .collect();
                    let _literal = Literal::new(get_id());
                    assert!(args.len() == 2 || (symbol == NOT && args.len() == 1));
                    match symbol.as_str() {
                        AND => and(args, clauses),
                        OR => or(args, clauses),
                        NOT => return args[0].not(),
                        IMPLICATION => implication(args, clauses),
                        EQUALITY => equality(args, clauses),
                        XOR => xor(args, clauses),
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn process_command(&mut self, command: Command) {
        if self.exit {
            return;
        }
        match command {
            Command::Assert { term } => {
                if self.no_logic() {
                    panic!("no logic")
                }
                let mut clauses = vec![];
                let literal = self.parse_term(term, &mut clauses);
                clauses.push(Clause::new(vec![literal]));
                self.assertion_sets.last_mut().unwrap().add_clauses(clauses);
            }
            Command::CheckSat => {
                self.solve();
            }
            Command::CheckSatAssuming { .. } => {}
            Command::DeclareConst { .. } => {}
            Command::DeclareDatatype { .. } => {}
            Command::DeclareDatatypes { .. } => {}
            Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => {
                let id = self.get_symbol_id(symbol);
                self.assertion_sets
                    .last_mut()
                    .unwrap()
                    .add_uninterpreted_function(id, parameters, sort);
            }
            Command::DeclareSort {
                symbol: _,
                arity: _,
            } => {}
            Command::DefineFun { sig: _, term: _ } => {}
            Command::DefineFunRec { .. } => {}
            Command::DefineFunsRec { .. } => {}
            Command::DefineSort { .. } => {}
            Command::Echo { message } => {
                println!("{}", message)
            }
            Command::Exit => self.exit = true,
            Command::GetAssertions => {}
            Command::GetAssignment => {}
            Command::GetInfo { .. } => {}
            Command::GetModel => {}
            Command::GetOption { .. } => {}
            Command::GetProof => {}
            Command::GetUnsatAssumptions => {}
            Command::GetUnsatCore => {}
            Command::GetValue { .. } => {}
            Command::Pop { level } => {
                if self.no_logic() {
                    panic!("Pop before set logic")
                }
                if numeral_larger_than_usize(&level) {
                    panic!("Pop level too large")
                }
                let level = level.to_u64_digits()[0] as usize;
                let current_len = self.assertion_sets.len();
                if level + 1 > current_len {
                    panic!("Pop bottom level")
                }
                self.assertion_sets.truncate(current_len - level);
            }
            Command::Push { level } => {
                if self.no_logic() {
                    panic!("Push before set logic")
                }
                if numeral_larger_than_usize(&level) {
                    panic!("Push level too large")
                }
                self.assertion_sets
                    .extend((0..level.to_u64_digits()[0]).map(|_| AssertionSet::default()));
            }
            Command::Reset => {}
            Command::ResetAssertions => {}
            Command::SetInfo { .. } => {}
            Command::SetLogic {
                symbol: Symbol(symbol),
            } => self.set_logic(symbol),
            Command::SetOption { keyword, value } => self.set_option(keyword, value),
        }
    }

    fn set_logic(&mut self, symbol: String) {
        if !self.no_logic() {
            panic!("Set logic twice")
        }
        self.logic = Some(Logic::new(&symbol));
    }

    fn get_symbol_id(&mut self, symbol: Symbol) -> usize {
        for set in self.assertion_sets.iter().rev() {
            if let Some(id) = set.get_id(&symbol) {
                return id;
            }
        }
        let id = get_id();
        self.assertion_sets.last_mut().unwrap().set_id(symbol, id);
        id
    }

    fn set_option(&mut self, keyword: Keyword, value: AttributeValue) {
        let Keyword(keyword) = keyword;
        // boolean
        match keyword.as_str() {
            PRINT_SUCCESS | PRODUCE_MODELS => {
                let _boolean = match value {
                    AttributeValue::Symbol(Symbol(sym)) => str_to_bool(&sym),
                    _ => {
                        unimplemented!()
                    }
                };
                match keyword.as_str() {
                    PRINT_SUCCESS => self.print_success = true,
                    PRODUCE_MODELS => self.produce_models = true,
                    _ => {
                        unimplemented!()
                    }
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn get_clauses(&self) -> Vec<Clause> {
        self.assertion_sets
            .iter()
            .map(|a| a.get_clauses())
            .flatten()
            .map(Clone::clone)
            .collect()
    }
}

fn str_to_bool(sym: &str) -> bool {
    match sym {
        "false" => false,
        "true" => true,
        _ => unimplemented!(),
    }
}

fn numeral_larger_than_usize(num: &Numeral) -> bool {
    num.bits() > std::mem::size_of::<usize>() as u64
}
