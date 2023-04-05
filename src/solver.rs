use std::collections::{HashMap, HashSet};
use log::info;
use crate::assertion_set::Clause;

pub struct SATSolver {
    ids: Vec<usize>,
    clauses: Vec<Clause>,
    pub assignments: Vec<Option<bool>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Res {
    SAT,
    UNSAT,
}

impl SATSolver {
    fn bcp(&mut self) {

    }

    fn decide(&mut self) {

    }

    pub fn solve(&mut self) -> Res {
        return self.solve_i(0)
    }

    pub fn solve_i(&mut self, cur: usize) -> Res {
        if cur == self.assignments.len() {
            return Res::SAT
        }
        self.assignments[cur] = Some(true);
        if self.clauses.iter().any(|c| !self.check_clause(c)) {
            return Res::UNSAT
        }
        let next = cur + 1;
        if self.solve_i(next) == Res::SAT {
            Res::SAT
        } else {
            self.assignments[cur] = Some(false);
            return self.solve_i(next)
        }
    }

    pub fn check_clause(&self, clause: &Clause) -> bool {
        clause.literals.iter()
            .any(|i| self.assignments[i.id]
                .map(|a| a == i.value)
                .unwrap_or(true))
    }

    pub fn new(mut clauses: Vec<Clause>) -> Self {
        let mut assignments: HashSet<_> = clauses
            .iter()
            .map(|c| c.literals.iter())
            .flatten()
            .map(|l| l.id).collect();

        let mut ids: Vec<_> = assignments.into_iter().collect();
        ids.sort();
        let len = ids.len();
        let id_to_rank: HashMap<_, _> = ids.iter().enumerate().map(|(rank, id)| (*id, rank)).collect();
        clauses.iter_mut().map(|c| c.literals.iter_mut()).flatten().for_each(|l| {
            l.id = *id_to_rank.get(&l.id).unwrap();
        });
        clauses.iter().for_each(|c| info!("{}", c));
        Self {
            ids,
            clauses,
            assignments: vec![None; len],
        }
    }
}

