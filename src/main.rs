pub mod board;
pub mod solver;

use solver::generate;

fn main() {
    print!("{}", generate(3).board());
}
