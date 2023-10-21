use _my_rayon::my_quicksort;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("quicksort 1", |b| {
        b.iter(|| {
            let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
            let mut big_input = [0_u32; 2_048 * 2];
            rng.fill(&mut big_input);

            my_quicksort(black_box(&mut big_input))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
