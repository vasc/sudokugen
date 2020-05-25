use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sudokugen::generate;
use sudokugen::solve;
fn solve_benchmark(c: &mut Criterion) {
    let table = ".724..3........49.........2921...5.7..4.6...3......2...4..7.....3..196....5..4.21"
        .parse()
        .unwrap();

    c.bench_function("solve", |b| b.iter(|| solve(black_box(&table)).unwrap()));
}

fn generate_benchmark(c: &mut Criterion) {
    c.bench_function("generate", |b| b.iter(|| generate(black_box(3))));
}

criterion_group!(solve_bench, solve_benchmark);
criterion_group!(
    name = gen_bench;
    config = Criterion::default().sample_size(40);
    targets = generate_benchmark
);

criterion_main!(solve_bench, gen_bench);
