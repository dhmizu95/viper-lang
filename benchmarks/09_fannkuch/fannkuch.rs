// Benchmark 09: Fannkuch
// Category: Discrete Mathematics / Permutations
// Tests: Array manipulation, permutations, recursion

use std::time::Instant;

const N: usize = 10;

static mut MAX_FLIPS: usize = 0;
static mut CHECKSUM: i32 = 0;

// Flip array elements up to index k
fn flip(arr: &mut [i32], k: usize) {
    let mut i = 0;
    let mut j = k;
    while i < j {
        arr.swap(i, j);
        i += 1;
        j -= 1;
    }
}

// Calculate fannkuch for a permutation
fn fannkuch(arr: &[i32], n: usize) -> usize {
    let mut temp = arr.to_vec();
    let mut flips = 0;

    while temp[0] != 0 {
        let idx = temp[0] as usize;
        flip(&mut temp, idx);
        flips += 1;
    }

    flips
}

// Generate permutations and calculate fannkuch
fn permute(arr: &mut [i32], count: &mut [usize], n: usize, depth: usize) {
    if depth == n {
        let flips = fannkuch(arr, n);

        unsafe {
            if flips > MAX_FLIPS {
                MAX_FLIPS = flips;
            }

            // Add to checksum with alternating sign
            let sign = if count[0] % 2 == 0 { 1 } else { -1 };
            CHECKSUM += sign * flips as i32;
        }

        return;
    }

    for i in depth..n {
        // Swap
        arr.swap(depth, i);

        count[depth] += 1;

        permute(arr, count, n, depth + 1);

        // Rotate back
        let temp = arr[depth];
        for j in depth..n - 1 {
            arr[j] = arr[j + 1];
        }
        arr[n - 1] = temp;

        if count[depth] >= n - depth {
            count[depth] = 0;
        } else {
            break;
        }
    }
}

fn main() {
    let start = Instant::now();

    let mut arr: [i32; N] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut count: [usize; N] = [0; N];

    permute(&mut arr, &mut count, N, 0);

    let elapsed = start.elapsed();

    unsafe {
        println!("Permutations of {} elements", N);
        println!("Maximum flips: {}", MAX_FLIPS);
        println!("Checksum: {}", CHECKSUM);
    }
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
