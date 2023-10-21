use _my_rayon::{fibonacci_1, fibonacci_2};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_fibs(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fibonacci");
    for i in [30u64, 31u64].iter() {
        group.bench_with_input(BenchmarkId::new("recur", i), i, |b, i| {
            b.iter(|| fibonacci_1(*i))
        });
        group.bench_with_input(BenchmarkId::new("iter", i), i, |b, i| {
            b.iter(|| fibonacci_2(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);
