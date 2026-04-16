//! FHE Performance Profiling and Benchmarking
//!
//! Provides utilities to measure and report execution metrics for FHE operations.
//! Essential for analyzing noise growth, latency, and hardware acceleration benefits.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};


/// Detailed measurement of an FHE operation benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub op_name: String,
    pub iterations: u32,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

/// A robust FHE performance profiler.
pub struct FheProfiler;

impl FheProfiler {
    /// Benchmarks any closure representing an FHE circuit or operation.
    ///
    /// # Arguments
    /// * `name`       - Descriptive name of the operation.
    /// * `iterations` - Number of times to run the operation.
    /// * `f`          - The FHE logic to benchmark.
    pub fn benchmark<F, R>(name: &str, iterations: u32, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        let mut min_dur = Duration::from_secs(3600); // Very high initial value
        let mut max_dur = Duration::from_secs(0);
        let mut total_dur = Duration::from_secs(0);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = f();
            let elapsed = start.elapsed();

            total_dur += elapsed;
            if elapsed < min_dur {
                min_dur = elapsed;
            }
            if elapsed > max_dur {
                max_dur = elapsed;
            }
        }

        let total_ms = total_dur.as_secs_f64() * 1000.0;

        BenchmarkResult {
            op_name: name.to_string(),
            iterations,
            total_duration_ms: total_ms,
            avg_duration_ms: total_ms / (iterations as f64),
            min_duration_ms: min_dur.as_secs_f64() * 1000.0,
            max_duration_ms: max_dur.as_secs_f64() * 1000.0,
        }
    }

    /// Prints a  performance report to the console.
    pub fn print_report(results: &[BenchmarkResult]) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                        🚀 FHESTATE PERFORMANCE METRICS REPORT                            ║");
        println!("╠══════════════════════════════════════╦════════════╦══════════════╦═══════════╦═══════════╣");
        println!("║ Operation Name                       ║ Iters      ║ Total (ms)   ║ Avg (ms)  ║ Max (ms)  ║");
        println!("╠══════════════════════════════════════╬════════════╬══════════════╬═══════════╬═══════════╣");

        for res in results {
            println!(
                "║ {:<36} ║ {:<10} ║ {:<12.2} ║ {:<9.2} ║ {:<9.2} ║",
                res.op_name,
                res.iterations,
                res.total_duration_ms,
                res.avg_duration_ms,
                res.max_duration_ms
            );
        }

        println!("╚══════════════════════════════════════╩════════════╩══════════════╩═══════════╩═══════════╝\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::FheMath;
    use crate::voting::VotingTally;
    use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint32};
    use tfhe::prelude::*;

    #[test]
    #[ignore = "Slow FHE keygen — run with: cargo test -- --ignored"]
    fn benchmark_dao_tally() {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);

        let vote_count = 8;
        let votes: Vec<FheUint32> = (0..vote_count)
            .map(|_| FheUint32::encrypt(1u32, &client_key))
            .collect();

        let mut results = Vec::new();

        // 1. Linear Tally Benchmark (O(n))
        results.push(FheProfiler::benchmark("Linear Tally (8-way)", 1, || {
            let mut sum = votes[0].clone();
            for i in 1..vote_count {
                sum = sum + &votes[i as usize];
            }
            sum
        }));

        // 2. Tree-Sum Tally Benchmark (O(log n) - Optimized)
        results.push(FheProfiler::benchmark("Tree-Sum Tally (8-way)", 1, || {
            FheMath::tree_sum(votes.clone()).unwrap()
        }));

        // 3. Black-Box Winner Detection Benchmark
        results.push(FheProfiler::benchmark(
            "Winner Detection (3 Candidates)",
            1,
            || {
                let candidates = vec![
                    FheUint32::encrypt(10u32, &client_key),
                    FheUint32::encrypt(42u32, &client_key),
                    FheUint32::encrypt(25u32, &client_key),
                ];
                VotingTally::find_winner(&candidates).unwrap()
            },
        ));

        FheProfiler::print_report(&results);
    }
}
