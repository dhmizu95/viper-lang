use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn run_bench(args: &BenchArgs) -> Result<(), String> {
    let benchmark_dir = "benchmark";

    if !Path::new(benchmark_dir).exists() {
        return Err(format!("Benchmark directory '{}' not found", benchmark_dir));
    }

    let entries = fs::read_dir(benchmark_dir)
        .map_err(|e| format!("Failed to read benchmark directory: {}", e))?;

    let mut benchmarks: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().map(|ext| ext == "vp").unwrap_or(false)
        })
        .collect();

    benchmarks.sort_by_key(|e| e.file_name());

    if let Some(ref file) = args.file {
        let path = Path::new(benchmark_dir).join(file);
        if !path.exists() {
            return Err(format!("Benchmark file '{}' not found", file));
        }
        run_single_benchmark(&path, args.iterations)?;
    } else {
        println!("Running {} benchmarks...\n", benchmarks.len());

        for entry in &benchmarks {
            run_single_benchmark(&entry.path(), args.iterations)?;
        }

        println!("\nAll benchmarks complete.");
    }

    Ok(())
}

fn run_single_benchmark(path: &Path, iterations: u32) -> Result<(), String> {
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

    print!("{}: ", name);

    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;

    let mut total_time = 0.0;

    for i in 0..iterations {
        let start = Instant::now();

        // Parse tokens only
        let mut lexer = crate::lexer::Lexer::new(&source);
        let _tokens = lexer.tokenize()?;

        total_time += start.elapsed().as_secs_f64();

        if i < iterations - 1 {
            print!(".");
        }
    }

    let avg_time = total_time / iterations as f64;
    println!(" {:.3}s (avg of {} runs)", avg_time, iterations);

    Ok(())
}

pub struct BenchArgs {
    pub file: Option<String>,
    pub iterations: u32,
}

impl BenchArgs {
    pub fn new(file: Option<String>, iterations: u32) -> Self {
        Self { file, iterations }
    }
}
