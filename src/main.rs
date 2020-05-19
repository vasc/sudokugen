pub mod board;
pub mod solver;

// use solver::generate;
use sudoku_generator::solver::generate;

fn main() {
    // for _ in 0..10000 {
    //     // print!("{}", generate(3).board());

    //     let table: Board =
    //         ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21"
    //             .parse()
    //             .unwrap();

    //     solve(&table).unwrap();
    // }

    for _ in 0..1000 {
        generate(3);
    }
}
