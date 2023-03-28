pub mod constants;
pub mod assertion_set;
pub mod solver;

use std::sync::atomic::{AtomicUsize, Ordering};
pub use solver::Solver;

fn get_id() -> usize {
    static COUNTER:AtomicUsize = AtomicUsize::new(1);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    if id == 0 {
        panic!("ID overflow")
    }
    id
}
