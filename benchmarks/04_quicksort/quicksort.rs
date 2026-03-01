// Benchmark 04: QuickSort
// Category: Discrete Mathematics / Sorting
// Tests: Recursion, array manipulation, comparisons

use std::time::Instant;

const SIZE: usize = 100_000; // 100k elements

fn partition(arr: &mut [i32], low: usize, high: usize) -> usize {
    let pivot = arr[high];
    let mut i = low as i32 - 1;

    for j in low..high {
        if arr[j] <= pivot {
            i += 1;
            arr.swap(i as usize, j);
        }
    }
    arr.swap((i + 1) as usize, high);
    (i + 1) as usize
}

fn quick_sort(arr: &mut [i32], low: usize, high: usize) {
    if low < high {
        let pi = partition(arr, low, high);
        if pi > 0 {
            quick_sort(arr, low, pi - 1);
        }
        quick_sort(arr, pi + 1, high);
    }
}

fn main() {
    let start = Instant::now();

    // Allocate and initialize array
    let mut arr: Vec<i32> = (0..SIZE).map(|i| ((SIZE - i) * 17 % SIZE) as i32).collect();

    // Sort
    quick_sort(&mut arr, 0, SIZE - 1);

    let elapsed = start.elapsed();

    // Verify sorted
    let sorted = arr.windows(2).all(|w| w[0] <= w[1]);

    println!("Array size: {}", SIZE);
    println!("Sorted correctly: {}", sorted);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
