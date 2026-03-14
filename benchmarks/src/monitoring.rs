use std::process::Command;
use std::path::Path;
use std::fs;
use std::time::Instant;

pub struct MemoryMonitor {
    pid: u32,
}

impl MemoryMonitor {
    pub fn new(pid: u32) -> Self {
        MemoryMonitor { pid }
    }

    pub fn get_memory_usage_mb(&self) -> f64 {
        if cfg!(target_os = "linux") {
            let output = Command::new("cat")
                .arg(format!("/proc/{}/status", self.pid))
                .output()
                .ok();

            if let Some(output) = output {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].parse::<f64>().unwrap_or(0.0) / 1024.0;
                        }
                    }
                }
            }
        }
        0.0
    }
}

pub struct CPUMonitor {
    pid: u32,
}

impl CPUMonitor {
    pub fn new(pid: u32) -> Self {
        CPUMonitor { pid }
    }

    pub fn get_cpu_usage_percent(&self) -> f64 {
        if cfg!(target_os = "linux") {
            let output = Command::new("top")
                .arg("-b")
                .arg("-n")
                .arg("1")
                .arg("-p")
                .arg(self.pid.to_string())
                .output()
                .ok();

            if let Some(output) = output {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines().skip(7) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 9 && parts[0] == self.pid.to_string() {
                        return parts[8].parse::<f64>().unwrap_or(0.0);
                    }
                }
            }
        }
        0.0
    }
}

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn new() -> Self {
        BatteryMonitor
    }

    pub fn get_battery_consumption_mwh(&self) -> f64 {
        if cfg!(target_os = "linux") {
            let output = Command::new("cat")
                .arg("/sys/class/power_supply/BAT0/capacity")
                .output()
                .ok();

            if let Some(output) = output {
                let capacity = String::from_utf8_lossy(&output.stdout).trim().parse::<f64>().unwrap_or(0.0);
                return (100.0 - capacity) * 10.0;
            }
        }
        0.0
    }

    pub fn start_monitoring(&self) -> BatterySession {
        BatterySession {
            start_capacity: self.get_battery_consumption_mwh(),
            start_time: Instant::now(),
        }
    }
}

pub struct BatterySession {
    start_capacity: f64,
    start_time: Instant,
}

impl BatterySession {
    pub fn get_consumption(&self, monitor: &BatteryMonitor) -> f64 {
        let end_capacity = monitor.get_battery_consumption_mwh();
        end_capacity - self.start_capacity
    }

    pub fn get_duration_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

pub struct PerformanceProfiler {
    memory_monitor: Option<MemoryMonitor>,
    cpu_monitor: Option<CPUMonitor>,
    battery_monitor: Option<BatteryMonitor>,
}

impl PerformanceProfiler {
    pub fn new(pid: Option<u32>) -> Self {
        PerformanceProfiler {
            memory_monitor: pid.map(MemoryMonitor::new),
            cpu_monitor: pid.map(CPUMonitor::new),
            battery_monitor: Some(BatteryMonitor::new()),
        }
    }

    pub fn collect_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            memory_usage_mb: self.memory_monitor.as_ref()
                .map(|m| m.get_memory_usage_mb())
                .unwrap_or(0.0),
            cpu_usage_percent: self.cpu_monitor.as_ref()
                .map(|c| c.get_cpu_usage_percent())
                .unwrap_or(0.0),
            battery_consumption_mwh: self.battery_monitor.as_ref()
                .map(|b| b.get_battery_consumption_mwh())
                .unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub battery_consumption_mwh: f64,
}

pub struct SystemInfo;

impl SystemInfo {
    pub fn get_cpu_info() -> String {
        if cfg!(target_os = "linux") {
            let output = Command::new("cat")
                .arg("/proc/cpuinfo")
                .output()
                .ok();

            if let Some(output) = output {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.starts_with("model name") {
                        return line.split(':').nth(1).unwrap_or("Unknown").trim().to_string();
                    }
                }
            }
        }
        "Unknown CPU".to_string()
    }

    pub fn get_memory_info() -> String {
        if cfg!(target_os = "linux") {
            let output = Command::new("free")
                .arg("-h")
                .output()
                .ok();

            if let Some(output) = output {
                let content = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() > 1 {
                    return lines[1].to_string();
                }
            }
        }
        "Unknown Memory".to_string()
    }

    pub fn get_os_info() -> String {
        if cfg!(target_os = "linux") {
            let output = Command::new("uname")
                .arg("-a")
                .output()
                .ok();

            if let Some(output) = output {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        "Unknown OS".to_string()
    }
}