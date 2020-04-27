use criterion::{black_box, criterion_group, criterion_main, Criterion};
// extern crate sudoku_generator;
use sudoku_generator::board::Board;
use sudoku_generator::solver::solve;

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n - 1) + fibonacci(n - 2),
//     }
// }

fn solve_benchmark(c: &mut Criterion) {
    let table = Board::from(
        ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21",
    );

    c.bench_function("solve", |b| b.iter(|| solve(black_box(&table)).unwrap()));
}

// fn find_minimal_map(c: &mut Criterion) {
//     c.bench_function("find_minimal_board", |b| {
//         let mut table = Board::new(3);
//         b.iter(|| {
//             let solver.
//             table.solve().unwrap();
//             table.find_minimal_board();
//         })
//     });
// }

criterion_group!(benches, solve_benchmark);
criterion_main!(benches);
