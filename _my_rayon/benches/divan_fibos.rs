fn main() {
    //? Run registered benchmarks.
    divan::main();
}

use std::time::Duration;

#[divan::bench_group(max_time = Duration::from_secs(1))]
mod fibonacci {
    use _my_rayon::{fibonacci_1, fibonacci_2};

    const N: u64 = 40;

    #[divan::bench]
    fn iterative() -> u64 {
        let mut previous = 1;
        let mut current = 1;

        for _ in 2..=divan::black_box(N) {
            let next = previous + current;
            previous = current;
            current = next;
        }

        current
    }
    #[divan::bench]
    fn iterative_2() -> u64 {
        fibonacci_2(divan::black_box(N))
    }

    #[divan::bench]
    fn recursive() -> u64 {
        fn compute(n: u64) -> u64 {
            if n <= 1 {
                1
            } else {
                compute(n - 2) + compute(n - 1)
            }
        }

        compute(divan::black_box(N))
    }

    #[divan::bench]
    fn recursive_2() -> u64 {
        fibonacci_1(divan::black_box(N))
    }
}
