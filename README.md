# Rusmt

## Abstract

Rusmt is a CDCL(conflict-driven clause learning) SAT solver. It is a Rust implementation of [GRASP](https://www.cs.cmu.edu/~emc/15-820A/reading/grasp_iccad96.pdf). It can solve 3-SAT problems with 2000 variables and clauses in less than 15s. The core CDCL code line count is about 400. 

## Naming

The initial goal of this project is to build a SMT solver(in Rust) that supports linear integer arithmetic theory. However, I failed to defeat [z3](https://www.cs.cmu.edu/~emc/15-820A/reading/grasp_iccad96.pdf) in SAT solving.

## Structure
- SMT-LIB context front end and SAT solver back end
- Hash map for assignments
- Use Rust iterators heavily to simplify code logic

## Algorithms

### Data structure
Assignment: variable id, value, parent clause id, decision level

### Pseudocode

1. Preprocessing: remove unary clauses
2. Clause minimization: remove redundant literals in clauses
3. Compute the frequencies of literals and save them in a map  
4. Select the highest-frequency literal whose variable is unassigned and assign it a boolean value that makes the literal true(parent clause id to None, this is a decision node). If no such literal is found, all variables are assigned and no conflict found, return `SAT`.
5. Enter BCP(boolean constraint propagation) subroutine loop. 
6. Check all clauses for conflicts, once a conflict is found, return the clause id; if no conflict, search all clauses for unit clauses, do assignments(set parent clause to the unit clause) and run 6 again until no unit clause is found.
7. Exit BCP loop
8. If no conflict, go to 4;
9. else:
   1. traverse the implication graph(using the parent clause id in Assignment) 
   2. collect all decision nodes that cause this conflict as roots
   3. Find the highest level unflipped root and flip its assignment, set the current decision level to this level and undo all assignments(except this root) whose decision level is higher than or equal to this level. If no unflipped root exists, return UNSAT. 
   4. go to 4

## Features
- Accepts SAT problem in [SMT-LIB](http://smtlib.cs.uiowa.edu/) format.
  - Convert the parsing tree into [CNF](https://en.wikipedia.org/wiki/Conjunctive_normal_form) with [Tseitin encoding](https://en.wikipedia.org/wiki/Tseytin_transformation)
- Preprocessing
  - [Clause minimization](http://minisat.se/downloads/escar05.pdf): delete unnecessary literals in a clause
  - Keep searching and eliminating unary clauses until no unary clauses left
- Random test case generation
  - Generate literals randomly and combine them to make clauses
  - Run z3 and this solver on the same test case
  - Can solve 3-SAT problem with ~1000 literals and clauses in less than 15s. Still 100x times slower than z3.
- Potential improvement:
  - Watched literals
  - Use u32 as ID type, more cache friendly
  - Better heuristics

