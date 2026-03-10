// QuickSort Benchmark - Rust Implementation
// Array sorting algorithm with iterative quicksort

fn partition(arr: &mut [i64], low: usize, high: usize) -> usize {
    let pivot = arr[high];
    let mut i = low;
    
    for j in low..high {
        if arr[j] < pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, high);
    i
}

fn quicksort(arr: &mut [i64], mut low: usize, mut high: usize) {
    // Use stack to avoid recursion
    let mut stack = [0usize; 100];
    let mut top = 0;
    
    if low >= high {
        return;
    }
    
    stack[top] = low;
    top += 1;
    stack[top] = high;
    top += 1;
    
    while top > 0 {
        top -= 1;
        let h = stack[top];
        top -= 1;
        let l = stack[top];
        
        if l < h {
            let p = partition(arr, l, h);
            
            if p > l {
                stack[top] = l;
                top += 1;
                stack[top] = p - 1;
                top += 1;
            }
            
            if p + 1 < h {
                stack[top] = p + 1;
                top += 1;
                stack[top] = h;
                top += 1;
            }
        }
    }
}

fn main() {
    // Benchmark parameter - array size
    let n = 100;
    
    // Initialize array with pseudo-random values
    let mut arr = vec![0i64; n];
    let mut seed = 12345i64;
    
    for i in 0..n {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        arr[i] = seed % 1000;
    }
    
    // Sort array
    quicksort(&mut arr, 0, n - 1);
    
    // Calculate checksum to verify sort
    let checksum: i64 = arr.iter().sum();
    println!("quicksort {} elements checksum: {}", n, checksum);
}
