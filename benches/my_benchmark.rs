use criterion::{black_box, criterion_group, criterion_main, Criterion};
extern crate sudoku_generator;
use sudoku_generator::board::Board;

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n - 1) + fibonacci(n - 2),
//     }
// }

fn criterion_benchmark(c: &mut Criterion) {
    let table = Board::from(
        ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21",
    );

    c.bench_function("solve", |b| b.iter(|| table.clone().solve()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
