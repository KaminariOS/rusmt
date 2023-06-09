pub mod assertion_set;
pub mod cli;
pub mod constants;
pub mod context;
pub mod solver;
#[cfg(test)]
mod test;

use std::sync::atomic::{AtomicUsize, Ordering};

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    if id == 0 {
        panic!("ID overflow")
    }
    id
}
