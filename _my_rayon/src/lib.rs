#[inline]
pub fn fibonacci_1(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci_1(n - 1) + fibonacci_1(n - 2),
    }
}

pub fn fibonacci_2(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

#[inline]
pub fn my_quicksort<T: PartialOrd + std::fmt::Debug>(input: &mut [T]) {
    /*
     * Base Case:
     *
     * If the input list arr has 1 or fewer elements,
     * it is already sorted, so the function returns the list as is.
     *
     * if len(arr) <= 1: return arr
     */
    if input.len() <= 1 {
        return;
    }
    /*
     * Partitioning the List:
     *
     * The input list is partitioned into two sublists: less and greater.
     * less contains elements less than or equal to the pivot,
     * and greater contains elements greater than the pivot.
     *
     * List comprehensions are used to create these sublists:
     * - less = [x for x in arr[1:] if x <= pivot]
     * - greater = [x for x in arr[1:] if x > pivot]
     */
    let mid = {
        let pivot = input.len() - 1;
        let mut i = 0;
        for j in 0..pivot {
            if input[j] <= input[pivot] {
                input.swap(i, j);
                i += 1;
            }
        }
        input.swap(i, pivot);
        i
    };

    let (lo, hi) = input.split_at_mut(mid);
    // println!("lo: {lo:?}, hi: {hi:?}");
    /*
     * Combining Sorted Sublists:
     *
     * The sorted less list, pivot element,
     * and sorted greater list are concatenated together.
     *
     * return quick_sort(less) + [pivot] + quick_sort(greater)
     */
    my_quicksort(lo);
    my_quicksort(hi);
}

pub fn functional_quicksort(input: &mut [u32]) {
    if input.len() <= 1 {
        return;
    }

    // let pivot = 4;

    // let mut partitioned: Vec<i32> = input().drain(..).partition(|&x| x <= pivot).2.collect();

    // input.append(&mut partitioned);

    // println!("Partitioned: {:?}", input);

    // let (lo, hi) = input.split_at_mut(mid);

    // functional_quicksort(lo);
    // functional_quicksort(hi);
}
#[cfg(test)]
mod test {
    use crate::*;
    use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};

    #[test]
    fn sorts() {
        let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

        let mut big_input = [0_u32; 2_048 * 2];
        rng.fill(&mut big_input);

        my_quicksort(&mut big_input);

        let mut sorted_data = big_input;
        sorted_data.sort();

        big_input.iter().take(10).for_each(|x| {
            println!("{x}");
        });
        assert_eq!(big_input, sorted_data);
    }
}
