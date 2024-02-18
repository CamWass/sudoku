use criterion::{criterion_group, criterion_main, Criterion};
use data::*;
use solver::intuitive::solve;
use solver::*;

fn test_solve(puzzle: [u8; 81]) -> Board {
    let input = Board {
        inner: std::hint::black_box(puzzle),
    };

    solve(input, false)
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("solve");

    group.bench_with_input("puzzle1", &INPUT1, |b, input| {
        b.iter(|| test_solve(*input));
    });
    group.bench_with_input("puzzle2", &INPUT2, |b, input| {
        b.iter(|| test_solve(*input));
    });
    group.bench_with_input("puzzle3", &INPUT3, |b, input| {
        b.iter(|| test_solve(*input));
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
