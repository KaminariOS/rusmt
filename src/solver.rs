use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use log::info;
// use rayon::prelude::*;
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
use strum_macros::{Display, AsRefStr};
#[derive(PartialEq, Eq)]
#[derive(Display, AsRefStr)]
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
    assignments: HashMap<usize, Assignment>,
    decision_nodes: Vec<usize>,
}

// pub fn preprocess(clauses: Vec<Clause>, assignments:&mut Vec<Option<Assignment>>) -> Vec<Clause> {
//     let mut cs = Vec::with_capacity(clauses.len());
//     for clause in clauses {
//         if clause
//     }
// }
#[derive(Debug)]
pub enum PropogationResult {
    Unit(Vec<(Literal, usize)>),
    Conflict(usize)
}

impl CDCLSolver {
    pub fn get_assignments(&self) -> Vec<(usize, bool)> {
        self.ids.iter()
            .zip(self.assignments.iter())
            .map(|(id, (lid, a))| (*id, a.value) )
            .collect()
    }
}

impl CDCLSolver {
    pub fn new(clause: Vec<Clause>) -> Self {
        let (ids, clauses) = rename(clause);
        let len = ids.len();
        let assignments= HashMap::with_capacity(len);
        // let clauses = preprocess(clauses, &mut assignments);
        Self {
            ids,
            clauses,
            assignments,
            decision_nodes: vec![0],
        }
    }

    pub fn fully_assign(&self) -> bool {
        self.assignments.len() == self.ids.len()
    }

    pub fn solve(&mut self) -> Res {
        let mut current_decision_level = 0;
        let mut current_variable = 0;
        let total_variable = self.ids.len();

        // if let Some(core) = self.propagation(current_decision_level) {
        //     return Res::UNSAT;
        // }
        while !self.fully_assign() {
            if self.assignments.contains_key(&current_variable) {
                current_variable = (current_variable + 1) % total_variable;
                continue;
            }
            current_decision_level += 1;
            self.decision_nodes.push(current_variable);
            assert_eq!(self.decision_nodes.len(), current_decision_level + 1);
            self.assignments.insert(current_variable,
                                    Assignment::new(false, None, current_decision_level)
            );
            loop {
                match self.propagation() {
                    PropogationResult::Unit(literals) => {
                        if literals.is_empty() {
                            break
                        }
                        literals.into_iter().for_each(|(l, i)|
                            {
                                // assert!(self.assignments[l.id].is_none());
                                self.assignments.insert(l.id, Assignment {
                                    value: l.value,
                                    clause: Some(i),
                                    decision_level: current_decision_level,
                                });
                            }
                        );
                        // info!("Unit: {}; level: {}", id, current_decision_level)
                    }
                    PropogationResult::Conflict(core) => {
                        info!("conflict: {}; len: {}", core, self.clauses.len());
                        let mut roots = vec![];
                        self.collect_roots(&mut roots, core);
                        roots.dedup();
                        let conflict_clause: Vec<_> = roots.iter().map(|&r|
                            {
                                Literal {
                                    value: !self.assignments[&r].value,
                                    id: r
                                }
                            }
                        ).collect();
                        self.clauses.push(Clause { literals: conflict_clause });
                        let mut root_levels: Vec<_> = roots.iter().map(|&r| self.assignments[&r].decision_level).collect();
                        root_levels.sort();
                        let mut backtrack_decision_level = None;
                        while let Some(highest_level) = root_levels.pop() {
                            let highest_conflict_decision = self.decision_nodes[highest_level];
                            let decision_node = self.assignments.get_mut(&highest_conflict_decision).unwrap();
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
                        let tobe_removed: Vec<_> = self.assignments
                            .iter()
                            .filter(|(_, a)|
                                    a.decision_level >= current_decision_level
                                )
                            .map(|(i, _)| *i)
                            .collect();
                        tobe_removed.into_iter().for_each(|i| {
                            self.assignments
                                .remove(&i);
                        });
                        self.assignments.insert(self.decision_nodes[current_decision_level], Assignment {
                            value: true,
                            clause: None,
                            decision_level: current_decision_level,
                        });
                        self.decision_nodes = self.decision_nodes[0..=current_decision_level].to_vec();
                    }
                }
            }

        }
        println!("{:?}", self.propagation());
        assert!(self.fully_assign());
        SAT
    }

    pub fn collect_roots(&self, roots: &mut Vec<usize>, clause: usize) {
        let next: Vec<_> = self.clauses[clause].literals.iter().filter_map(|l|
            {
                if let Some(c) = self.assignments[&l.id].clause {
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

    pub fn propagation(&mut self) -> PropogationResult {
        let c_len = self.clauses.len();
        for index in 0..c_len {
            let conflict = !self.clauses[index].literals.iter()
                .any(|i|
                    self.assignments.get(&i.id)
                    .map(|a| a.value == i.value)
                        .unwrap_or(true)
                );
            if conflict {
                return PropogationResult::Conflict(index)
            }
        }

        let mut units: Vec<_> = self.clauses.iter()
            .enumerate()
            // .par_bridge()
            .filter_map(|(i, c)|
                 {
                     let unresolved: Vec<_> = c.literals
                         .iter()
                         .filter(|l| !self.assignments.contains_key(&l.id)).collect();

                     let diff: Vec<_> = c.literals
                         .iter()
                         .filter_map(|l| self.assignments.get(&l.id).filter(|a| a.value != l.value))
                         .dedup()
                         .collect();
                     // .filter(|l| l.value != ).collect();
                     if unresolved.len() == 1 && diff.len() + 1 == c.len() {
                         Some((*unresolved[0], i))
                     } else {
                         None
                     }
                 }
            )
            .collect();
        units.dedup();
        // units.truncate(1);
        PropogationResult::Unit(units)
    }
}

