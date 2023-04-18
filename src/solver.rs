use crate::assertion_set::{Clause, Literal};
use crate::solver::Res::{SAT, UNSAT};
use itertools::Itertools;
use log::info;
use std::collections::{HashMap, HashSet};

pub struct SATSolver {
    ids: Vec<usize>,
    clauses: Vec<Clause>,
    pub assignments: Vec<Option<bool>>,
}

impl SATSolver {
    pub fn get_assignments(&self) -> Vec<(usize, Option<bool>)> {
        self.ids
            .iter()
            .zip(self.assignments.iter())
            .map(|a| (*a.0, *a.1))
            .collect()
    }
}
use strum_macros::{AsRefStr, Display};
#[derive(PartialEq, Eq, Display, AsRefStr, Copy, Clone)]
// If we don't care about inner capitals, we don't need to set `serialize_all`
// and can leave parenthesis empty.
#[strum(serialize_all = "snake_case")]
pub enum Res {
    SAT,
    UNSAT,
}

impl SATSolver {
    pub fn solve(&mut self) -> Res {
        return self.solve_i(0);
    }

    pub fn solve_i(&mut self, cur: usize) -> Res {
        if cur == self.assignments.len() {
            return SAT;
        }
        self.assignments[cur] = Some(false);
        let no_conflict = self.clauses.iter().all(|c| self.check_clause(c));
        let next = cur + 1;
        let res = if no_conflict && self.solve_i(next) == SAT {
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
        clause
            .literals
            .iter()
            .any(|i| self.assignments[i.id].map(|a| a == i.value).unwrap_or(true))
    }

    pub fn new(clauses: Vec<Clause>) -> Self {
        let (ids, clauses) = rename(clauses);
        // clauses.iter().for_each(|c| info!("{}", c.display(&ids)));
        let len = ids.len();
        Self {
            ids,
            clauses,
            assignments: vec![None; len],
        }
    }
}
pub fn rename(mut clauses: Vec<Clause>) -> (Vec<usize>, Vec<Clause>) {
    let mut ids: Vec<_> = clauses
        .iter()
        .map(|c| c.literals.iter())
        .flatten()
        .map(|l| l.id)
        .unique()
        .collect();

    ids.sort();
    let id_to_rank: HashMap<_, _> = ids
        .iter()
        .enumerate()
        .map(|(rank, id)| (*id, rank))
        .collect();
    clauses.iter_mut().for_each(|c| {
        c.literals = c
            .literals
            .iter()
            .map(|l| {
                let mut new_l = l.clone();
                new_l.id = id_to_rank[&l.id];
                new_l
            })
            .collect();
    });

    (ids, clauses)
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
struct Assignment {
    value: bool,
    clause: Option<usize>,
    decision_level: usize,
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
    frequency: HashMap<Literal, usize>,
    res: Option<Res>,
    watcher: HashMap<Literal, HashSet<usize>>
}

// pub fn preprocess(clauses: Vec<Clause>, assignments:&mut Vec<Option<Assignment>>) -> Vec<Clause> {
//     let mut cs = Vec::with_capacity(clauses.len());
//     for clause in clauses {
//         if clause
//     }
// }

impl CDCLSolver {
    pub fn get_assignments(&self) -> Vec<(usize, bool)> {
        self.ids
            .iter()
            .zip(self.assignments.iter())
            .map(|(id, (_lid, a))| (*id, a.value))
            .collect()
    }
}

// pub fn preprocess(clauses: Vec<Clause>) -> Option<Vec<Clause>> {
//     let mut assignments = HashMap::new();
//     loop {
//         let len = clauses.len();
//         let mut new_clauses = Vec::with_capacity(len);
//         for clause in clauses {
//             if clause.literals.len() == 1 {
//
//             }
//         }
//     }
// }

pub fn find_contradiction(clauses: &[Clause]) -> Option<Res> {
    let mut res = if clauses
        .iter()
        .any(|c| c.len() == 2 && c.literals.iter().any(|l| c.literals.contains(&l.not())))
    {
        Some(UNSAT)
    } else {
        None
    };
    if res.is_none() {
        let set: HashSet<_> = clauses
            .iter()
            .filter(|c| c.len() == 1)
            .map(|c| c.literals.iter())
            .flatten()
            .collect();
        if set.iter().any(|l| set.contains(&l.not())) {
            res = Some(UNSAT)
        }
    }
    res
}

pub fn remove_unary(mut clauses: Vec<Clause>) -> (Option<Res>, Vec<Clause>) {
    let mut assignments: HashMap<usize, bool> = HashMap::new();
    loop {
        let res = find_contradiction(&clauses);
        if res.is_some() {
            return (res, vec![]);
        }
        let (unarys, non_unary): (Vec<_>, Vec<_>) = clauses.into_iter().partition(|c| c.len() == 1);

        unarys
            .iter()
            .map(|c| c.literals.iter())
            .flatten()
            .for_each(|l| {
                assert!(!assignments.contains_key(&l.id));
                assignments.insert(l.id, l.value);
            });
        clauses = non_unary
            .into_iter()
            .filter_map(|mut c| {
                let mut literals = HashSet::new();
                for l in &c.literals {
                    if let Some(&value) = assignments.get(&l.id) {
                        if l.value == value {
                            return None;
                        }
                    } else {
                        literals.insert(*l);
                    }
                }
                if literals.is_empty() {
                    None
                } else {
                    c.literals = literals;
                    Some(c)
                }
            })
            .collect();
        if unarys.is_empty() {
            return (res, clauses);
        }
    }
}

pub fn watch_map(clauses: &Vec<Clause>) -> HashMap<Literal, HashSet<usize>> {
    let mut map: HashMap<Literal, HashSet<usize>> = HashMap::new();
    clauses.iter().enumerate().for_each(|(i, c)| {
        c.literals.iter().for_each(|l| {
            map.entry(*l).or_default().insert(i);
        })
    });
    map
}

fn minimize_cur_clauses(
    clauses: &[Clause],
    watch_map: &HashMap<Literal, HashSet<usize>>,
) -> Vec<Clause> {
    clauses
        .iter()
        .filter_map(|c| {
            let new_c = clause_minimization(c.clone(), clauses, watch_map);
            if new_c.literals.is_empty() {
                None
            } else {
                Some(new_c)
            }
        })
        .collect()
}

fn clause_minimization(
    mut clause: Clause,
    clauses: &[Clause],
    watch_map: &HashMap<Literal, HashSet<usize>>,
) -> Clause {
    let mut removable = HashSet::with_capacity(clause.len());
    for l in clause.literals.iter() {
        let not_l = l.not();
        if let Some(map) = watch_map.get(&not_l) {
            for &c_index in map {
                let c = &clauses[c_index];
                if c.literals
                    .iter()
                    .filter(|&&nl| nl != not_l)
                    .all(|nl| clause.literals.contains(nl))
                {
                    removable.insert(*l);
                    break;
                }
            }
        }
    }
    // info!("C_len: {}; removable len: {}", clause.len(), removable.len());
    clause.literals = clause.literals.difference(&removable).map(|x| *x).collect();
    clause
}

impl CDCLSolver {
    pub fn new(clauses: Vec<Clause>) -> Self {
        println!("Initial clauses: {}", clauses.len());
        let (res, mut clauses) = remove_unary(clauses);
        println!("Clauses after removing unary: {}", clauses.len());
        if res.is_none() {
            let watch_list = watch_map(&clauses);
            clauses = minimize_cur_clauses(&clauses, &watch_list);
            println!("Clauses after minimization: {}", clauses.len());
        }
        let (ids, mut clauses) = rename(clauses);
        let len = ids.len();
        let assignments = HashMap::with_capacity(len);
        let mut frequency: HashMap<Literal, usize> = HashMap::with_capacity(len);
        let watcher = watch_map(&clauses);
        clauses
            .iter()
            .map(|c| c.literals.iter())
            .flatten()
            .for_each(|l| *frequency.entry(*l).or_default() += 1);
        // let clauses = preprocess(clauses, &mut assignments);
        Self {
            ids,
            clauses,
            assignments,
            decision_nodes: vec![0],
            frequency,
            res,
            watcher,
        }
    }

    pub fn fully_assign(&self) -> bool {
        self.assignments.len() == self.ids.len()
    }

    pub fn get_next(&self) -> Option<Literal> {
        let mut freq: Vec<_> = self.frequency.iter()
            .filter(|(l, _)| !self.assignments.contains_key(&l.id))
            .collect();
        freq.sort_by_key(|x| (x.1, x.0.id, x.0.value));
        if let Some((l, _)) = freq.pop() {
                return Some(*l);
        }
        None
    }

    pub fn solve(&mut self) -> Res {
        // self.minimize_cur_clause();
        // self.clauses = self.clauses.iter().unique().collect();
        if let Some(res) = &self.res {
            return *res;
        }
        let mut current_decision_level = 0;

        while let Some(cur) = self.get_next() {
            let current_variable = cur.id;
            current_decision_level += 1;
            self.decision_nodes.push(current_variable);
            assert_eq!(self.decision_nodes.len(), current_decision_level + 1);
            self.assignments.insert(
                current_variable,
                Assignment::new(cur.value, None, current_decision_level),
            );
            loop {
                if let Some(core) = self.propagation(current_decision_level) {
                    info!("conflict: {}; len: {}", core, self.clauses.len());
                    let mut roots = HashSet::new();
                    let mut visited = HashSet::new();
                    self.collect_roots(&mut roots, core, &mut visited);
                    let conflict_clause: Vec<_> = roots
                        .iter()
                        .map(|&r| Literal {
                            value: !self.assignments[&r].value,
                            id: r,
                        })
                        .collect();
                    let conflict_clause = Clause::new(conflict_clause);
                    // conflict_clause = self.clause_minimization(conflict_clause);
                    if !conflict_clause.is_empty() {
                        conflict_clause
                            .literals
                            .iter()
                            .for_each(|l| {
                                *self.frequency.entry(*l).or_default() += 1;
                                let len = self.clauses.len();
                                self.watcher.get_mut(l).unwrap().insert(len);
                            });
                        self.clauses.push(conflict_clause);
                    }
                    let mut root_levels: Vec<_> = roots
                        .iter()
                        .map(|&r| self.assignments[&r].decision_level)
                        .collect();
                    root_levels.sort();
                    let mut backtrack_decision_level = None;
                    while let Some(highest_level) = root_levels.pop() {
                        let highest_conflict_decision = self.decision_nodes[highest_level];
                        let decision_node = self
                            .assignments
                            .get_mut(&highest_conflict_decision)
                            .unwrap();
                        if decision_node.value == cur.value {
                            backtrack_decision_level = Some(highest_level);
                            break;
                        }
                    }
                    if let Some(back) = backtrack_decision_level {
                        current_decision_level = back;
                    } else {
                        return UNSAT;
                    }
                    let tobe_removed: Vec<_> = self
                        .assignments
                        .iter()
                        .filter(|(_, a)| a.decision_level < current_decision_level)
                        .map(|(i, _)| *i)
                        .collect();
                    tobe_removed.into_iter().for_each(|i| {
                        self.assignments.remove(&i);
                    });
                    self.assignments.insert(
                        self.decision_nodes[current_decision_level],
                        Assignment {
                            value: !cur.value,
                            clause: None,
                            decision_level: current_decision_level,
                        },
                    );
                    self.decision_nodes = self.decision_nodes[0..=current_decision_level].to_vec();
                } else {
                    break;
                }
            }
        }
        // println!("{:?}", self.propagation());
        // println!("Assignment: {}/{}; freq: {}", self.assignments.len(), self.ids.len()
        //          , self.frequency.len());
        assert!(self.fully_assign());
        SAT
    }

    pub fn collect_roots(
        &self,
        roots: &mut HashSet<usize>,
        clause: usize,
        visited: &mut HashSet<usize>,
    ) {
        if visited.contains(&clause) {
            return;
        }
        let next: Vec<_> = self.clauses[clause]
            .literals
            .iter()
            .filter_map(|l| {
                if let Some(c) = self.assignments[&l.id].clause {
                    if c != clause {
                        Some(c)
                    } else {
                        None
                    }
                } else {
                    roots.insert(l.id);
                    None
                }
            })
            .collect();
        visited.insert(clause);
        next.into_iter()
            .for_each(|n| self.collect_roots(roots, n, visited));
    }

    pub fn propagation(&mut self, current_decision_level: usize) -> Option<usize> {
        let c_len = self.clauses.len();
        loop {
            for index in 0..c_len {
                let conflict = !self.clauses[index].literals.iter().any(|i| {
                    self.assignments
                        .get(&i.id)
                        .map(|a| a.value == i.value)
                        .unwrap_or(true)
                });
                if conflict {
                    return Some(index);
                }
            }

            let units: Vec<_> = self
                .clauses
                .iter()
                .enumerate()
                // .par_bridge()
                .filter_map(|(i, c)| {
                    let unresolved: Vec<_> = c
                        .literals
                        .iter()
                        .filter(|l| !self.assignments.contains_key(&l.id))
                        .collect();

                    let diff: Vec<_> = c
                        .literals
                        .iter()
                        .filter_map(|l| self.assignments.get(&l.id).filter(|a| a.value != l.value))
                        .unique()
                        .collect();
                    // .filter(|l| l.value != ).collect();
                    if unresolved.len() == 1 && diff.len() + 1 == c.len() {
                        Some((*unresolved[0], i))
                    } else {
                        None
                    }
                })
                .collect();
            if units.is_empty() {
                return None;
            }

            units.into_iter()
                // .unique()
                .for_each(|(l, i)| {
                // assert!(self.assignments[l.id].is_none());
                self.assignments.insert(
                    l.id,
                    Assignment {
                        value: l.value,
                        clause: Some(i),
                        decision_level: current_decision_level,
                    },
                );
            });
        }
    }
}
