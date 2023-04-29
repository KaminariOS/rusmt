# Rusmt

A conflict-driven clause learning SAT solver. This is a Rust implementation of [GRASP](https://www.cs.cmu.edu/~emc/15-820A/reading/grasp_iccad96.pdf). I choose Rust because of its zero-cost abstraction.

## Naming

The initial goal of this project is to build a SMT solver that supports linear integer arithmetic theory. However, I failed to defeat [z3](https://www.cs.cmu.edu/~emc/15-820A/reading/grasp_iccad96.pdf) in SAT solving.

## Structure
- SMT-LIB context front end and SAT solver back end
- Hash map for assignments
- Use Rust iterators heavily to simplify code logic

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

