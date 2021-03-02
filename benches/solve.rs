use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use sudokugen::{board::BoardSize, Board, Puzzle};

fn solve_benchmark(c: &mut Criterion) {
    let table: Board =
        ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21"
            .parse()
            .unwrap();

    c.bench_function("solve", |b| {
        b.iter_batched(
            || table.clone(),
            |mut table| table.solve(),
            BatchSize::SmallInput,
        )
    });
}

fn generate_benchmark(c: &mut Criterion) {
    c.bench_function("generate", |b| {
        b.iter(|| Puzzle::generate(black_box(BoardSize::NineByNine)))
    });
}

criterion_group!(solve_bench, solve_benchmark);
criterion_group!(
    name = gen_bench;
    config = Criterion::default().sample_size(40);
    targets = generate_benchmark
);

criterion_main!(solve_bench, gen_bench);
