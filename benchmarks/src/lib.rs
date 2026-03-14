use std::time::{Duration, Instant};
use std::process::Command;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub version: String,
    pub targets: HashMap<String, TargetConfig>,
    pub metrics: MetricsConfig,
    pub scenarios: ScenariosConfig,
    pub reporting: ReportingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub path: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub startup_time: MetricConfig,
    pub memory_usage: MetricConfig,
    pub execution_speed: MetricConfig,
    pub cpu_usage: MetricConfig,
    pub battery_consumption: MetricConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    pub enabled: bool,
    pub unit: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenariosConfig {
    pub javascript_execution: ScenarioConfig,
    pub module_loading: ScenarioConfig,
    pub file_io: ScenarioConfig,
    pub network_requests: ScenarioConfig,
    pub ai_agent_execution: ScenarioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub enabled: bool,
    pub iterations: u32,
    pub warmup: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub output_dir: String,
    pub format: Vec<String>,
    pub include_charts: bool,
    pub comparison_target: String,
    pub performance_goal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub target: String,
    pub scenario: String,
    pub metric: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub scenario: String,
    pub metric: String,
    pub zeo_value: f64,
    pub bun_value: f64,
    pub improvement_ratio: f64,
    pub goal_met: bool,
    pub unit: String,
}

pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        BenchmarkRunner {
            config,
            results: Vec::new(),
        }
    }

    pub fn run_all(&mut self) -> Result<(), String> {
        println!("Running Zeo Benchmark Suite v{}", self.config.version);
        println!("========================================\n");

        for (target_name, _) in &self.config.targets {
            self.run_target_benchmarks(target_name)?;
        }

        Ok(())
    }

    fn run_target_benchmarks(&mut self, target: &str) -> Result<(), String> {
        println!("Benchmarking target: {}", target);
        println!("----------------------------\n");

        if self.config.scenarios.javascript_execution.enabled {
            self.run_startup_time_benchmark(target)?;
        }

        if self.config.scenarios.javascript_execution.enabled {
            self.run_javascript_execution_benchmark(target)?;
        }

        if self.config.scenarios.module_loading.enabled {
            self.run_module_loading_benchmark(target)?;
        }

        if self.config.scenarios.file_io.enabled {
            self.run_file_io_benchmark(target)?;
        }

        if self.config.scenarios.network_requests.enabled {
            self.run_network_benchmark(target)?;
        }

        if self.config.scenarios.ai_agent_execution.enabled {
            self.run_ai_agent_benchmark(target)?;
        }

        Ok(())
    }

    fn run_startup_time_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.javascript_execution;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running startup time benchmark ({} iterations, {} warmup)...", iterations, warmup);

        for i in 0..(warmup + iterations) {
            let start = Instant::now();
            
            let _output = Command::new(&self.config.targets[target].command)
                .arg("--version")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_millis() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "startup_time".to_string(),
                    metric: "startup_time".to_string(),
                    value: duration,
                    unit: "ms".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "startup_time")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average startup time: {:.2} ms", avg_duration);
        println!();

        Ok(())
    }

    fn run_javascript_execution_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.javascript_execution;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running JavaScript execution benchmark...");

        let test_code = r#"
            function fibonacci(n) {
                if (n <= 1) return n;
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
            fibonacci(30);
        "#;

        for i in 0..(warmup + iterations) {
            let start = Instant::now();

            let _output = Command::new(&self.config.targets[target].command)
                .arg("-e")
                .arg(test_code)
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_micros() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "javascript_execution".to_string(),
                    metric: "execution_speed".to_string(),
                    value: duration,
                    unit: "μs".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "javascript_execution")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average execution time: {:.2} μs", avg_duration);
        println!();

        Ok(())
    }

    fn run_module_loading_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.module_loading;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running module loading benchmark...");

        for i in 0..(warmup + iterations) {
            let start = Instant::now();

            let _output = Command::new(&self.config.targets[target].command)
                .arg("tests/modules/load.js")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_millis() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "module_loading".to_string(),
                    metric: "load_time".to_string(),
                    value: duration,
                    unit: "ms".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "module_loading")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average module load time: {:.2} ms", avg_duration);
        println!();

        Ok(())
    }

    fn run_file_io_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.file_io;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running file I/O benchmark...");

        for i in 0..(warmup + iterations) {
            let start = Instant::now();

            let _output = Command::new(&self.config.targets[target].command)
                .arg("tests/io/file_benchmark.js")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_millis() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "file_io".to_string(),
                    metric: "io_time".to_string(),
                    value: duration,
                    unit: "ms".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "file_io")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average file I/O time: {:.2} ms", avg_duration);
        println!();

        Ok(())
    }

    fn run_network_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.network_requests;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running network request benchmark...");

        for i in 0..(warmup + iterations) {
            let start = Instant::now();

            let _output = Command::new(&self.config.targets[target].command)
                .arg("tests/network/http_benchmark.js")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_millis() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "network_requests".to_string(),
                    metric: "request_time".to_string(),
                    value: duration,
                    unit: "ms".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "network_requests")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average network request time: {:.2} ms", avg_duration);
        println!();

        Ok(())
    }

    fn run_ai_agent_benchmark(&mut self, target: &str) -> Result<(), String> {
        let scenario = &self.config.scenarios.ai_agent_execution;
        let iterations = scenario.iterations;
        let warmup = scenario.warmup;

        println!("Running AI Agent execution benchmark...");

        for i in 0..(warmup + iterations) {
            let start = Instant::now();

            let _output = Command::new(&self.config.targets[target].command)
                .arg("tests/ai/agent_benchmark.js")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", target, e))?;

            let duration = start.elapsed().as_millis() as f64;

            if i >= warmup {
                let result = BenchmarkResult {
                    target: target.to_string(),
                    scenario: "ai_agent_execution".to_string(),
                    metric: "agent_execution_time".to_string(),
                    value: duration,
                    unit: "ms".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                };
                self.results.push(result);
            }
        }

        let avg_duration: f64 = self.results.iter()
            .filter(|r| r.target == target && r.scenario == "ai_agent_execution")
            .map(|r| r.value)
            .sum::<f64>() / iterations as f64;

        println!("  Average AI Agent execution time: {:.2} ms", avg_duration);
        println!();

        Ok(())
    }

    pub fn compare_with_bun(&self) -> Vec<ComparisonResult> {
        let mut comparisons = Vec::new();
        let comparison_target = &self.config.reporting.comparison_target;
        let goal = self.config.reporting.performance_goal;

        let scenarios = vec![
            "startup_time",
            "javascript_execution",
            "module_loading",
            "file_io",
            "network_requests",
            "ai_agent_execution",
        ];

        for scenario in &scenarios {
            let zeo_results: Vec<_> = self.results.iter()
                .filter(|r| r.target == "zeo" && r.scenario == *scenario)
                .collect();

            let bun_results: Vec<_> = self.results.iter()
                .filter(|r| r.target == comparison_target && r.scenario == *scenario)
                .collect();

            if !zeo_results.is_empty() && !bun_results.is_empty() {
                let zeo_avg: f64 = zeo_results.iter().map(|r| r.value).sum::<f64>() / zeo_results.len() as f64;
                let bun_avg: f64 = bun_results.iter().map(|r| r.value).sum::<f64>() / bun_results.len() as f64;
                
                let improvement_ratio = bun_avg / zeo_avg;
                let goal_met = improvement_ratio >= goal;

                comparisons.push(ComparisonResult {
                    scenario: scenario.to_string(),
                    metric: "performance".to_string(),
                    zeo_value: zeo_avg,
                    bun_value: bun_avg,
                    improvement_ratio,
                    goal_met,
                    unit: zeo_results[0].unit.clone(),
                });
            }
        }

        comparisons
    }

    pub fn get_results(&self) -> &Vec<BenchmarkResult> {
        &self.results
    }
}