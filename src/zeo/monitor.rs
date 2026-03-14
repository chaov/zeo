use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    pub count: u64,
    pub total_duration: Duration,
    pub total_memory: u64,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub min_memory: Option<u64>,
    pub max_memory: Option<u64>,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            total_memory: 0,
            min_duration: None,
            max_duration: None,
            min_memory: None,
            max_memory: None,
        }
    }
    
    pub fn avg_duration(&self) -> Duration {
        if self.count > 0 {
            self.total_duration / self.count as u32
        } else {
            Duration::ZERO
        }
    }
    
    pub fn avg_memory(&self) -> u64 {
        if self.count > 0 {
            self.total_memory / self.count
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub executions: HashMap<String, ExecutionMetrics>,
    pub start_time: Instant,
    pub last_update: Instant,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            executions: HashMap::new(),
            start_time: Instant::now(),
            last_update: Instant::now(),
        }
    }
    
    pub fn record_execution(&mut self, script_id: &str, duration: Duration, memory: u64) {
        let metrics = self.executions
            .entry(script_id.to_string())
            .or_insert_with(ExecutionMetrics::new);
        
        metrics.count += 1;
        metrics.total_duration += duration;
        metrics.total_memory += memory;
        
        metrics.min_duration = Some(
            metrics.min_duration
                .map_or(duration, |min| min.min(duration))
        );
        
        metrics.max_duration = Some(
            metrics.max_duration
                .map_or(duration, |max| max.max(duration))
        );
        
        metrics.min_memory = Some(
            metrics.min_memory
                .map_or(memory, |min| min.min(memory))
        );
        
        metrics.max_memory = Some(
            metrics.max_memory
                .map_or(memory, |max| max.max(memory))
        );
        
        self.last_update = Instant::now();
    }
    
    pub fn get_metrics(&self, script_id: &str) -> Option<&ExecutionMetrics> {
        self.executions.get(script_id)
    }
    
    pub fn all_metrics(&self) -> &HashMap<String, ExecutionMetrics> {
        &self.executions
    }
    
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

pub trait Sampler: Send + Sync {
    fn start(&self);
    fn stop(&self);
    fn sample(&self) -> Sample;
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub timestamp: Instant,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub disk_io: DiskIO,
    pub network_io: NetworkIO,
}

#[derive(Debug, Clone)]
pub struct DiskIO {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_count: u64,
    pub write_count: u64,
}

#[derive(Debug, Clone)]
pubstruct NetworkIO {
    pub sent_bytes: u64,
    pub received_bytes: u64,
    pub connections: u32,
}

pub struct CPUSampler {
    is_running: Arc<Mutex<bool>>,
}

impl CPUSampler {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Sampler for CPUSampler {
    fn start(&self) {
        *self.is_running.lock().unwrap() = true;
    }
    
    fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }
    
    fn sample(&self) -> Sample {
        let cpu_usage = if *self.is_running.lock().unwrap() {
            self.calculate_cpu_usage()
        } else {
            0.0
        };
        
        Sample {
            timestamp: Instant::now(),
            cpu_usage,
            memory_usage: 0,
            disk_io: DiskIO {
                read_bytes: 0,
                write_bytes: 0,
                read_count: 0,
                write_count: 0,
            },
            network_io: NetworkIO {
                sent_bytes: 0,
                received_bytes: 0,
                connections: 0,
            },
        }
    }
}

impl CPUSampler {
    fn calculate_cpu_usage(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/stat") {
                if let Some(line) = content.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 8 {
                        let user: u64 = parts[1].parse().unwrap_or(0);
                        let nice: u64 = parts[2].parse().unwrap_or(0);
                        let system: u64 = parts[3].parse().unwrap_or(0);
                        let idle: u64 = parts[4].parse().unwrap_or(0);
                        
                        let total = user + nice + system + idle;
                        if total > 0 {
                            return 100.0 * (1.0 - idle as f64 / total as f64);
                        }
                    }
                }
            }
        }
        
        0.0
    }
}

pub trait Reporter: Send + Sync {
    fn generate_report(&self, metrics: &PerformanceMetrics) -> PerformanceReport;
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub generated_at: Instant,
    pub uptime: Duration,
    pub total_executions: u64,
    pub avg_execution_time: Duration,
    pub avg_memory_usage: u64,
    pub bottlenecks: Vec<Bottleneck>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub severity: BottleneckSeverity,
    pub category: BottleneckCategory,
    pub description: String,
    pub impact: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckCategory {
    CPU,
    Memory,
    DiskIO,
    NetworkIO,
    ExecutionTime,
}

pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn generate_report(&self, metrics: &PerformanceMetrics) -> PerformanceReport {
        let total_executions: u64 = metrics.executions
            .values()
            .map(|m| m.count)
            .sum();
        
        let total_duration: Duration = metrics.executions
            .values()
            .map(|m| m.total_duration)
            .sum();
        
        let total_memory: u64 = metrics.executions
            .values()
            .map(|m| m.total_memory)
            .sum();
        
        let avg_execution_time = if total_executions > 0 {
            total_duration / total_executions as u32
        } else {
            Duration::ZERO
        };
        
        let avg_memory_usage = if total_executions > 0 {
            total_memory / total_executions
        } else {
            0
        };
        
        let bottlenecks = self.analyze_bottlenecks(metrics);
        let recommendations = self.generate_recommendations(&bottlenecks);
        
        PerformanceReport {
            generated_at: Instant::now(),
            uptime: metrics.uptime(),
            total_executions,
            avg_execution_time,
            avg_memory_usage,
            bottlenecks,
            recommendations,
        }
    }
}

impl ConsoleReporter {
    fn analyze_bottlenecks(&self, metrics: &PerformanceMetrics) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        for (script_id, exec_metrics) in metrics.executions.iter() {
            let avg_duration = exec_metrics.avg_duration();
            
            if avg_duration > Duration::from_millis(100) {
                bottlenecks.push(Bottleneck {
                    severity: BottleneckSeverity::High,
                    category: BottleneckCategory::ExecutionTime,
                    description: format!("Script '{}' has slow execution: {:?}", script_id, avg_duration),
                    impact: avg_duration.as_millis() as f64,
                });
            }
            
            let avg_memory = exec_metrics.avg_memory();
            if avg_memory > 10 * 1024 * 1024 {
                bottlenecks.push(Bottleneck {
                    severity: BottleneckSeverity::Medium,
                    category: BottleneckCategory::Memory,
                    description: format!("Script '{}' uses high memory: {} MB", script_id, avg_memory / 1024 / 1024),
                    impact: avg_memory as f64,
                });
            }
        }
        
        bottlenecks.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
        bottlenecks
    }
    
    fn generate_recommendations(&self, bottlenecks: &[Bottleneck]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for bottleneck in bottlenecks {
            match bottleneck.category {
                BottleneckCategory::ExecutionTime => {
                    recommendations.push("Consider optimizing script execution or enabling JIT compilation".to_string());
                }
                BottleneckCategory::Memory => {
                    recommendations.push("Consider reducing memory usage or implementing memory pooling".to_string());
                }
                BottleneckCategory::CPU => {
                    recommendations.push("Consider optimizing CPU-intensive operations or parallelizing work".to_string());
                }
                _ => {}
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is optimal. No immediate recommendations.".to_string());
        }
        
        recommendations
    }
}

pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    samplers: Vec<Box<dyn Sampler>>,
    reporters: Vec<Box<dyn Reporter>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            samplers: Vec::new(),
            reporters: vec![Box::new(ConsoleReporter)],
        }
    }
    
    pub fn add_sampler(&mut self, sampler: Box<dyn Sampler>) {
        self.samplers.push(sampler);
    }
    
    pub fn add_reporter(&mut self, reporter: Box<dyn Reporter>) {
        self.reporters.push(reporter);
    }
    
    pub fn record_execution(&self, script_id: &str, duration: Duration, memory: u64) {
        self.metrics.lock().unwrap().record_execution(script_id, duration, memory);
    }
    
    pub fn record_execution_sync(&self, script_id: &str, duration: Duration, memory: u64) {
        self.metrics.lock().unwrap().record_execution(script_id, duration, memory);
    }
    
    pub fn start_sampling(&self) {
        for sampler in &self.samplers {
            sampler.start();
        }
    }
    
    pub fn stop_sampling(&self) {
        for sampler in &self.samplers {
            sampler.stop();
        }
    }
    
    pub fn generate_report(&self) -> PerformanceReport {
        let metrics = self.metrics.lock().unwrap();
        let mut report = PerformanceReport {
            generated_at: Instant::now(),
            uptime: metrics.uptime(),
            total_executions: 0,
            avg_execution_time: Duration::ZERO,
            avg_memory_usage: 0,
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
        };
        
        for reporter in &self.reporters {
            report = reporter.generate_report(&metrics);
        }
        
        report
    }
    
    pub fn metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }
}