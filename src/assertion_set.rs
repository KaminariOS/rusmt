use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;
use log::info;
use smt2parser::concrete::{Sort, Symbol};
use crate::get_id;

struct Signature {
    parameters: Vec<Sort>,
    result: Sort
}

pub(crate) struct AssertionSet {
    uninterpreted_functions: HashMap<usize, Signature>,
    symbol_table: HashMap<usize, Symbol>,
    symbol_table_rev: HashMap<Symbol, usize>, clauses: Vec<Clause>
}

impl AssertionSet {
    pub fn add_clauses(&mut self, clauses: Vec<Clause>) {
        self.clauses.extend(clauses);
    }
}

#[derive(Default, Clone)]
pub struct Clause {
    pub(crate) literals: Vec<Literal>
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.literals)
    }
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self {literals}
    }
}

#[derive(Copy, Clone)]
pub struct Literal {
    pub(crate) value: bool,
    pub(crate) id: usize
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = if self.value {
            ""
        } else {
            "^"
        };
        write!(f, "{}{}", s, self.id)
    }
}

pub fn and(args: Vec<Literal>, clauses: &mut Vec<Clause>) -> Literal {
    let literal = Literal::new(get_id());
    args.into_iter().for_each(|l| clauses.push(Clause::new(vec![literal.not(), l])));
    literal
}


pub fn implication(args: Vec<Literal>, clauses: &mut Vec<Clause>) -> Literal {
    let literal = Literal::new(get_id());
    clauses.push(Clause::new(vec![literal.not(), args[0].not(), args[1]]));
    literal
}

pub fn equality(args: Vec<Literal>, clauses: &mut Vec<Clause>) -> Literal {
    let backward = implication(vec![args[1], args[0]], clauses);
    let forward = implication(args, clauses);
    and(vec![forward, backward], clauses)
}


pub fn xor(mut args: Vec<Literal>, clauses: &mut Vec<Clause>) -> Literal {
    args[0] = args[0].not();
    equality(args, clauses)
}

pub fn or(mut args: Vec<Literal>, clauses: &mut Vec<Clause>) -> Literal {
    let literal = Literal::new(get_id());
    args.push(literal.not());
    clauses.push(Clause::new(args));
    literal
}


impl Literal {
    pub fn new(id: usize) -> Self {
        Self {
            value: true,
            id
        }
    }
    pub fn not(&self) -> Self {
        Self {
            value: !self.value,
            id: self.id
        }
    }
}

impl AssertionSet {
    pub fn add_uninterpreted_function(&mut self, symbol_id: usize, parameters: Vec<Sort>, result: Sort) {
        self.uninterpreted_functions.insert(symbol_id, Signature {parameters, result});
    }

    pub fn get_id(&self, symbol: &Symbol) -> Option<usize> {
        self.symbol_table_rev.get(&symbol).map(|&id| id)
    }

    pub fn set_id(&mut self, symbol: Symbol, id: usize) {
        assert!(!self.symbol_table.contains_key(&id) && !self.symbol_table_rev.contains_key(&symbol));
        self.symbol_table.insert(id, symbol.clone());
        self.symbol_table_rev.insert(symbol, id);
    }

    pub fn get_clauses(&self) -> Iter<Clause> {
        self.clauses.iter()
    }
}

impl Default for AssertionSet {
    fn default() -> Self {
        Self {
            uninterpreted_functions: HashMap::new(),
            symbol_table: Default::default(),
            symbol_table_rev: Default::default(),
            clauses: vec![],
        }
    }
}

type SolverID= usize;




