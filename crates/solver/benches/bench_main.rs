use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data::*;
use solver::*;

fn test_solve(puzzle: [u8; 81]) {
    let input = Board { inner: puzzle };

    let board = solve(input, false);

    black_box(&board);
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve");

    group.bench_with_input("puzzle1", &INPUT1, |b, input| {
        b.iter(|| {
            black_box(test_solve(*input));
        });
    });
    group.bench_with_input("puzzle2", &INPUT2, |b, input| {
        b.iter(|| {
            black_box(test_solve(*input));
        });
    });
    group.bench_with_input("puzzle3", &INPUT3, |b, input| {
        b.iter(|| {
            black_box(test_solve(*input));
        });
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
