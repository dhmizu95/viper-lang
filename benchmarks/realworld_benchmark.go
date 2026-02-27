// Viper vs Go - Real World Task Benchmark
// Go implementation for comparison with Viper's fiber scheduler
//
// Run with: go run realworld_benchmark.go
// Run with GOMAXPROCS: GOMAXPROCS=4 go run realworld_benchmark.go

package main

import (
	"fmt"
	"math/rand"
	"runtime"
	"sync"
	"sync/atomic"
	"time"
)

const (
	MatrixSize       = 256
	HashIterations   = 10000
	PipelineStages   = 5
	PipelineItems    = 1000
)

// getMemoryStats returns current memory usage
func getMemoryStats() runtime.MemStats {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	return m
}

// Benchmark 1: Matrix Multiplication
func benchmarkMatrixMultiply(numMatrices int) float64 {
	fmt.Println("\n╔════════════════════════════════════════╗")
	fmt.Println("║  Benchmark 1: Matrix Multiplication   ║")
	fmt.Println("╚════════════════════════════════════════╝")
	fmt.Printf("Matrices: %d, Size: %dx%d\n", numMatrices, MatrixSize, MatrixSize)

	var startMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&startMem)

	// Allocate matrices
	matricesA := make([][]float64, numMatrices)
	matricesB := make([][]float64, numMatrices)
	results := make([][]float64, numMatrices)

	for i := 0; i < numMatrices; i++ {
		matricesA[i] = make([]float64, MatrixSize*MatrixSize)
		matricesB[i] = make([]float64, MatrixSize*MatrixSize)
		results[i] = make([]float64, MatrixSize*MatrixSize)

		for j := range matricesA[i] {
			matricesA[i][j] = rand.Float64()
			matricesB[i][j] = rand.Float64()
		}
	}

	var matrixTasksDone int64
	var wg sync.WaitGroup

	start := time.Now()

	// Submit matrix multiplication tasks
	for i := 0; i < numMatrices; i++ {
		for row := 0; row < MatrixSize; row++ {
			wg.Add(1)
			go func(matrixA, matrixB, result []float64, row, size int) {
				defer wg.Done()

				for j := 0; j < size; j++ {
					sum := 0.0
					for k := 0; k < size; k++ {
						sum += matrixA[row*size+k] * matrixB[k*size+j]
					}
					result[row*size+j] = sum
				}

				atomic.AddInt64(&matrixTasksDone, 1)
			}(matricesA[i], matricesB[i], results[i], row, MatrixSize)
		}
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds() * 1000

	var endMem runtime.MemStats
	runtime.ReadMemStats(&endMem)

	// Verify results
	var checksum float64
	for i := 0; i < numMatrices; i++ {
		checksum += results[i][0]
	}

	fmt.Printf("Time: %.2f ms\n", elapsed)
	fmt.Printf("Throughput: %.0f matrix-rows/sec\n", float64(numMatrices*MatrixSize)/(elapsed/1000.0))
	fmt.Printf("Memory: Alloc %d KB -> %d KB (+%d KB)\n",
		startMem.Alloc/1024, endMem.Alloc/1024, (endMem.Alloc-startMem.Alloc)/1024)
	fmt.Printf("Checksum: %.6f\n", checksum)
	fmt.Printf("Tasks completed: %d\n", atomic.LoadInt64(&matrixTasksDone))

	return elapsed
}

// Benchmark 2: Hash Computation
func hashCompute(data uint64, seed uint64) uint64 {
	hash := uint64(14695981039346656037) ^ seed
	for i := 0; i < 8; i++ {
		hash ^= (data >> (i * 8)) & 0xFF
		hash *= 1099511628211
	}
	return hash
}

func benchmarkHashComputation(numHashes int) float64 {
	fmt.Println("\n╔════════════════════════════════════════╗")
	fmt.Println("║  Benchmark 2: Hash Computation        ║")
	fmt.Println("╚════════════════════════════════════════╝")
	fmt.Printf("Hashes: %d, Iterations: %d\n", numHashes, HashIterations)

	var startMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&startMem)

	type hashTask struct {
		seed       uint64
		iterations int
		result     uint64
	}

	tasks := make([]hashTask, numHashes)
	for i := range tasks {
		tasks[i].seed = rand.Uint64()
		tasks[i].iterations = HashIterations
	}

	var hashTasksDone int64
	var wg sync.WaitGroup

	start := time.Now()

	for i := 0; i < numHashes; i++ {
		wg.Add(1)
		go func(task *hashTask) {
			defer wg.Done()

			result := task.seed
			for j := 0; j < task.iterations; j++ {
				result = hashCompute(result, uint64(j))
			}
			task.result = result

			atomic.AddInt64(&hashTasksDone, 1)
		}(&tasks[i])
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds() * 1000

	var endMem runtime.MemStats
	runtime.ReadMemStats(&endMem)

	var checksum uint64
	for _, task := range tasks {
		checksum += task.result
	}

	fmt.Printf("Time: %.2f ms\n", elapsed)
	fmt.Printf("Throughput: %.0f hashes/sec\n", float64(numHashes)/(elapsed/1000.0))
	fmt.Printf("Memory: Alloc %d KB -> %d KB (+%d KB)\n",
		startMem.Alloc/1024, endMem.Alloc/1024, (endMem.Alloc-startMem.Alloc)/1024)
	fmt.Printf("Checksum: %d\n", checksum)
	fmt.Printf("Tasks completed: %d\n", atomic.LoadInt64(&hashTasksDone))

	return elapsed
}

// Benchmark 3: Producer-Consumer Pipeline
func benchmarkPipeline(numPipelines int) float64 {
	fmt.Println("\n╔════════════════════════════════════════╗")
	fmt.Println("║  Benchmark 3: Producer-Consumer       ║")
	fmt.Println("╚════════════════════════════════════════╝")
	fmt.Printf("Pipelines: %d, Stages: %d, Items: %d\n",
		numPipelines, PipelineStages, PipelineItems)

	var startMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&startMem)

	type pipelineTask struct {
		data          []int64
		stage         int
		stageCounters []int64
	}

	pipelineData := make([][]int64, numPipelines)
	stageCounters := make([][]int64, numPipelines)

	for p := 0; p < numPipelines; p++ {
		pipelineData[p] = make([]int64, PipelineItems)
		stageCounters[p] = make([]int64, PipelineStages)

		for i := range pipelineData[p] {
			pipelineData[p][i] = int64(i)
		}
	}

	var pipelineItemsProcessed int64
	var wg sync.WaitGroup

	start := time.Now()

	for p := 0; p < numPipelines; p++ {
		for s := 0; s < PipelineStages; s++ {
			wg.Add(1)
			go func(data []int64, stage int, counters []int64) {
				defer wg.Done()

				for i := range data {
					data[i] = data[i]*2 + int64(stage)
				}

				counters[stage] += int64(len(data))
				atomic.AddInt64(&pipelineItemsProcessed, int64(len(data)))
			}(pipelineData[p], s, stageCounters[p])
		}
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds() * 1000

	var endMem runtime.MemStats
	runtime.ReadMemStats(&endMem)

	var checksum int64
	for p := 0; p < numPipelines; p++ {
		for _, v := range pipelineData[p] {
			checksum += v
		}
	}

	fmt.Printf("Time: %.2f ms\n", elapsed)
	fmt.Printf("Throughput: %.0f pipeline-stages/sec\n",
		float64(numPipelines*PipelineStages)/(elapsed/1000.0))
	fmt.Printf("Items processed: %d\n", atomic.LoadInt64(&pipelineItemsProcessed))
	fmt.Printf("Memory: Alloc %d KB -> %d KB (+%d KB)\n",
		startMem.Alloc/1024, endMem.Alloc/1024, (endMem.Alloc-startMem.Alloc)/1024)
	fmt.Printf("Checksum: %d\n", checksum)

	return elapsed
}

// Benchmark 4: Web Scraper Simulation
func benchmarkWebScraper(numURLs int) float64 {
	fmt.Println("\n╔════════════════════════════════════════╗")
	fmt.Println("║  Benchmark 4: Web Scraper Simulation  ║")
	fmt.Println("╚════════════════════════════════════════╝")
	fmt.Printf("URLs: %d\n", numURLs)

	var startMem runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&startMem)

	type scraperTask struct {
		urlID        int
		url          string
		responseSize int
		result       int64
	}

	tasks := make([]scraperTask, numURLs)
	for i := range tasks {
		tasks[i].urlID = i
		tasks[i].url = "http://example.com/page"
		tasks[i].responseSize = 1024
	}

	var scraperTasksDone int64
	var wg sync.WaitGroup

	start := time.Now()

	for i := 0; i < numURLs; i++ {
		wg.Add(1)
		go func(task *scraperTask) {
			defer wg.Done()

			// Simulate network delay
			var sum int64
			for j := 0; j < 1000; j++ {
				sum += int64(j) * int64(task.urlID)
			}

			task.result = sum
			atomic.AddInt64(&scraperTasksDone, 1)
		}(&tasks[i])
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds() * 1000

	var endMem runtime.MemStats
	runtime.ReadMemStats(&endMem)

	var checksum int64
	for _, task := range tasks {
		checksum += task.result
	}

	fmt.Printf("Time: %.2f ms\n", elapsed)
	fmt.Printf("Throughput: %.0f URLs/sec\n", float64(numURLs)/(elapsed/1000.0))
	fmt.Printf("Memory: Alloc %d KB -> %d KB (+%d KB)\n",
		startMem.Alloc/1024, endMem.Alloc/1024, (endMem.Alloc-startMem.Alloc)/1024)
	fmt.Printf("Checksum: %d\n", checksum)
	fmt.Printf("Tasks completed: %d\n", atomic.LoadInt64(&scraperTasksDone))

	return elapsed
}

func runFullBenchmark() {
	fmt.Println()
	fmt.Println("╔══════════════════════════════════════════════════════════╗")
	fmt.Println("║         GO GOROUTINES - REAL WORLD BENCHMARKS           ║")
	fmt.Println("╚══════════════════════════════════════════════════════════╝")
	fmt.Println()
	fmt.Printf("Platform: Go Goroutines (GOMAXPROCS=%d)\n", runtime.GOMAXPROCS(0))
	fmt.Printf("CPU Cores: %d\n", runtime.NumCPU())
	fmt.Printf("Go Version: %s\n", runtime.Version())
	fmt.Println()

	var totalTime float64

	totalTime += benchmarkMatrixMultiply(10)
	totalTime += benchmarkHashComputation(10000)
	totalTime += benchmarkPipeline(100)
	totalTime += benchmarkWebScraper(10000)

	fmt.Println()
	fmt.Println("╔══════════════════════════════════════════════════════════╗")
	fmt.Println("║                    SUMMARY                               ║")
	fmt.Println("╚══════════════════════════════════════════════════════════╝")
	fmt.Printf("Total time: %.2f ms (%.2f seconds)\n", totalTime, totalTime/1000.0)

	var finalMem runtime.MemStats
	runtime.ReadMemStats(&finalMem)
	fmt.Printf("Final memory: Alloc %d KB, TotalAlloc %d KB\n",
		finalMem.Alloc/1024, finalMem.TotalAlloc/1024)
	fmt.Printf("NumGC: %d\n", finalMem.NumGC)
}

func main() {
	rand.Seed(time.Now().UnixNano())

	// Set GOMAXPROCS to number of CPUs
	runtime.GOMAXPROCS(runtime.NumCPU())

	runFullBenchmark()
}
