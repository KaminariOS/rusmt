use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use log::info;
use crate::assertion_set::{Clause, Literal};
use crate::solver::Res::{SAT, UNSAT};

pub struct SATSolver {
    ids: Vec<usize>,
    clauses: Vec<Clause>,
    pub assignments: Vec<Option<bool>>,
}

impl SATSolver {
    pub fn get_assignments(&self) -> Vec<(usize, Option<bool>)> {
        self.ids.iter()
            .zip(self.assignments.iter())
            .map(|a| (*a.0, *a.1) )
            .collect()
    }
}
use strum_macros::Display;
#[derive(PartialEq, Eq)]
#[derive(Display)]
// If we don't care about inner capitals, we don't need to set `serialize_all`
// and can leave parenthesis empty.
#[strum(serialize_all = "snake_case")]
pub enum Res {
    SAT,
    UNSAT,
}

impl SATSolver {

    pub fn solve(&mut self) -> Res {
        return self.solve_i(0)
    }

    pub fn solve_i(&mut self, cur: usize) -> Res {
        if cur == self.assignments.len() {
            return SAT
        }
        self.assignments[cur] = Some(false);
        let no_conflict = self.clauses.iter().all(|c| self.check_clause(c));
        let next = cur + 1;
        let res = if no_conflict && self.solve_i(next) == SAT  {
           SAT
        } else {
            self.assignments[cur] = Some(true);
            let no_conflict = self.clauses.iter().all(|c| self.check_clause(c));
            if no_conflict {
                self.solve_i(next)
            } else {
                UNSAT
            }
        };
        self.assignments[cur] = None;
        res
    }

    pub fn check_clause(&self, clause: &Clause) -> bool {
        clause.literals.iter()
            .any(|i| self.assignments[i.id]
                .map(|a| a == i.value)
                .unwrap_or(true))
    }

    pub fn new(clauses: Vec<Clause>) -> Self {
        let (ids, clauses) = rename(clauses);
        clauses.iter().for_each(|c| info!("{}", c.display(&ids)));
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
    clauses.iter_mut().map(|c|
                               {
                                   c.dedup();
                                   c.literals.iter_mut()
                               }
    ).flatten()
        .for_each(|l| {
        l.id = *id_to_rank.get(&l.id).unwrap();
    });
    (ids, clauses)
}

#[derive(Clone, PartialEq, Debug)]
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

    pub fn is_decision_node(&self) -> bool {
        self.clause.is_none()
    }
}

pub struct CDCLSolver {
    ids: Vec<usize>,
    pub clauses: Vec<Clause>,
    assignments: Vec<Option<Assignment>>,
    decision_nodes: Vec<usize>
}

// pub fn preprocess(clauses: Vec<Clause>, assignments:&mut Vec<Option<Assignment>>) -> Vec<Clause> {
//     let mut cs = Vec::with_capacity(clauses.len());
//     for clause in clauses {
//         if clause
//     }
// }
pub enum PropogationResult {
    Unit(usize),
    Conflict(usize)
}

impl CDCLSolver {
    pub fn get_assignments(&self) -> Vec<(usize, Option<bool>)> {
        self.ids.iter()
            .zip(self.assignments.iter())
            .map(|a| (*a.0, a.1.as_ref().map(|a| a.value)) )
            .collect()
    }
}

impl CDCLSolver {
    pub fn new(clause: Vec<Clause>) -> Self {
        let (ids, clauses) = rename(clause);
        let len = ids.len();
        let assignments= vec![None; len];
        // let clauses = preprocess(clauses, &mut assignments);
        Self {
            ids,
            clauses,
            assignments,
            decision_nodes: vec![0],
        }
    }

    pub fn solve(&mut self) -> Res {
        let mut current_decision_level = 0;
        let mut current_variable = 0;
        let total_variable = self.ids.len();

        // if let Some(core) = self.propagation(current_decision_level) {
        //     return Res::UNSAT;
        // }
        while current_variable < total_variable {
            if self.assignments[current_variable].is_some() {
                current_variable += 1;
                continue;
            }
            current_decision_level += 1;
            self.decision_nodes.push(current_variable);
            assert_eq!(self.decision_nodes.len(), current_decision_level + 1);
            self.assignments[current_variable] = Some(Assignment::new(false, None, current_decision_level));
            while let Some(res) = self.propagation(current_decision_level) {
                match res {
                    PropogationResult::Unit(id) => {
                        info!("Unit: {}; level: {}", id, current_decision_level)
                    }
                    PropogationResult::Conflict(core) => {
                        info!("conflict: {}; len: {}", core, self.clauses.len());
                        let mut roots = vec![];
                        self.collect_roots(&mut roots, core);
                        roots.dedup();
                        let conflict_clause: Vec<_> = roots.iter().map(|&r|
                            {
                                Literal {
                                    value: !self.assignments[r].as_ref().unwrap().value,
                                    id: r
                                }
                            }
                        ).collect();
                        self.clauses.push(Clause { literals: conflict_clause});
                        let mut root_levels: Vec<_> = roots.iter().map(|&r| self.assignments[r].as_ref().unwrap().decision_level).collect();
                        root_levels.sort();
                        let mut backtrack_decision_level = None;
                        while let Some(highest_level) = root_levels.pop() {
                            let highest_conflict_decision = self.decision_nodes[highest_level];
                            let decision_node = self.assignments[highest_conflict_decision].as_mut().unwrap();
                            if !decision_node.value {
                                backtrack_decision_level = Some(highest_level);
                                break;
                            }
                        }
                        if let Some(back) = backtrack_decision_level {
                            current_decision_level = back;
                        } else {
                            return UNSAT
                        }
                        // {
                        //     info!("Decision node: {:?}", decision_node);
                        //     if decision_node.value {
                        //         if highest_level == 1 {
                        //             return UNSAT
                        //         } else {
                        //             current_decision_level = highest_level - 1;
                        //         }
                        //     } else {
                        //         decision_node.value = true;
                        //         current_decision_level = highest_level;
                        //     }
                        // }
                        self.assignments
                            .iter_mut()
                            .filter(|a|
                                a.as_ref().filter(
                                |assignment| assignment.decision_level >= current_decision_level
                            ).is_some())
                            .for_each(|a| *a = None );
                        self.assignments[self.decision_nodes[current_decision_level]] = Some(Assignment{
                            value: true,
                            clause: None,
                            decision_level: current_decision_level,
                        });
                        self.decision_nodes = self.decision_nodes[0..=current_decision_level].to_vec();
                    }
                }

            }

        }
        SAT
    }

    pub fn collect_roots(&self, roots: &mut Vec<usize>, clause: usize) {
        let next: Vec<_> = self.clauses[clause].literals.iter().filter_map(|l|
            {
                if let Some(c) = self.assignments[l.id].as_ref().unwrap().clause {
                    if c != clause {

                        Some(c)
                    } else {None}
                } else {
                    roots.push(l.id);
                    None
                }
            }
        ).collect();

        next.into_iter().for_each(|n| self.collect_roots(roots, n));
    }

    pub fn propagation(&mut self, decision_level: usize) -> Option<PropogationResult> {
        let c_len = self.clauses.len();
        for index in 0..c_len {
            let conflict = !self.clauses[index].literals.iter()
                .any(|i|
                    self.assignments[i.id]
                        .as_ref()
                    .map(|a| a.value == i.value)
                        .unwrap_or(true)
                );
            if conflict {
                return Some(PropogationResult::Conflict(index))
            } else {
                let unresolved: Vec<_> = self.clauses[index].literals
                    .iter()
                    .filter(|l| self.assignments[l.id].is_none()).collect();

                let diff: Vec<_> = self.clauses[index].literals
                    .iter()
                    .filter_map(|l| self.assignments[l.id].as_ref().filter(|a| a.value != l.value))
                    .dedup()
                    .collect();
                // .filter(|l| l.value != ).collect();
                if unresolved.len() == 1 && diff.len() + 1 == self.clauses[index].len() {
                    let id = unresolved[0].id;
                    let value = unresolved[0].value;
                    self.assignments[id] = Some(Assignment::new(value, Some(index), decision_level));
                    return Some(PropogationResult::Unit(id))
                }
            }
        }
        None
    }
}

