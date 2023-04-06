use std::collections::{HashMap, HashSet};
use std::mem;
use log::info;
use crate::assertion_set::{Clause, Literal};
use crate::solver::Res::UNSAT;

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
        let (ids, clauses) = rename(clauses);
        clauses.iter().for_each(|c| info!("{}", c));
        let len = ids.len();
        Self {
            ids,
            clauses,
            assignments: vec![None; len],
        }
    }
}
pub fn rename(mut clauses: Vec<Clause>) -> (Vec<usize>, Vec<Clause>) {
    let ids: HashSet<_> = clauses
        .iter()
        .map(|c| c.literals.iter())
        .flatten()
        .map(|l| l.id).collect();

    let mut ids: Vec<_> = ids.into_iter().collect();
    ids.sort();
    let id_to_rank: HashMap<_, _> = ids.iter().enumerate().map(|(rank, id)| (*id, rank)).collect();
    clauses.iter_mut().map(|c| c.literals.iter_mut()).flatten().for_each(|l| {
        l.id = *id_to_rank.get(&l.id).unwrap();
    });
    clauses.iter().for_each(|c| info!("{}", c));
    (ids, clauses)
}

struct Assignment {
    value: bool,
    clause: Option<usize>,
    decision_level: usize
}

impl Assignment {
    pub fn new(value: bool, clause: Option<usize>, decision_level: usize) -> Self {
        Self {
            value,
            clause,
            decision_level,
        }
    }
}

struct CDCLSolver {
    ids: Vec<usize>,
    clauses: Vec<Clause>,
    assignments: Vec<Option<Assignment>>,
}

// pub fn preprocess(clauses: Vec<Clause>, assignments:&mut Vec<Option<Assignment>>) -> Vec<Clause> {
//     let mut cs = Vec::with_capacity(clauses.len());
//     for clause in clauses {
//         if clause
//     }
// }

impl CDCLSolver {
    pub fn new(clause: Vec<Clause>) -> Self {
        let (ids, clauses) = rename(clause);
        let len = ids.len();
        let mut assignments= vec![None; len];
        // let clauses = preprocess(clauses, &mut assignments);
        Self {
            ids,
            clauses,
            assignments,
        }
    }

    pub fn solve(&mut self) -> Res {
        let mut current_decision_level = 0;
        let mut current_variable = 0;
        let total_variable = self.ids.len();

        if let Some(core) = self.propagation(current_decision_level) {
            return Res::UNSAT;
        }
        while current_variable < total_variable {
            if self.assignments[current_variable].is_some() {
                current_variable += 1;
                continue;
            }
            current_decision_level += 1;
            self.assignments[current_variable] = Some(Assignment::new(true, None, current_decision_level));
            if let Some(core) = self.propagation(current_decision_level) {
                let mut roots = vec![];
                self.collect_roots(&mut roots, core);
                let conflict_clause: Vec<_> = roots.iter().map(|&r|
                    {
                        Literal {
                            value: !self.assignments[r].unwrap().value,
                            id: r
                        }
                    }
                ).collect();
                self.clauses.push(Clause { literals: conflict_clause});
                let highest_level = roots.iter().map(|&r| self.assignments[r].unwrap().decision_level).max().unwrap();
                self.assignments.iter_mut().for_each(|x| {
                    if x.filter(|a| a.decision_level >= highest_level) {
                        *x = None;
                    }
                } );
                current_decision_level = highest_level - 1;
            }

        }
        Res::UNSAT
    }

    pub fn collect_roots(&self, roots: &mut Vec<usize>, clause: usize) {
        let next: Vec<_> = self.clauses[clause].literals.iter().filter_map(|l|
            {
                if let Some(c) = self.assignments[l.id].as_ref().unwrap().clause {
                    Some(c)
                } else {
                    roots.push(l.id);
                    None
                }
            }
        ).collect();

        next.into_iter().for_each(|n| self.collect_roots(roots, n));
    }

    pub fn propagation(&mut self, decision_level: usize) -> Option<usize> {
        let c_len = self.clauses.len();
        for index in 0..clauses.len() {
            let conflict = self.clauses[index].literals.iter()
                .any(|i|
                    self.assignments[i.id]
                    .map(|a| a == i.value)
                        .unwrap_or(true)
                );
            if conflict {
                return Some(index)
            } else {
                let unresolved: Vec<_> = self.clauses[index].literals
                    .iter()
                    .filter(|l| self.assignments[l.id].is_none()).collect();
                if unresolved.len() == 1 {
                    let id = unresolved[0].id;
                    let value = unresolved[0].value;
                    mem::drop(unresolved);
                    self.assignments[id] = Some(Assignment::new(value, Some(index), decision_level));
                    return self.propagation(decision_level)
                }
            }
        }
        None
    }

}

