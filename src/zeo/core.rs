use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use crate::engine::{JSEngine, JSValue, Result, EngineError};
use crate::resource::ResourceManager;
use crate::monitor::PerformanceMonitor;

#[derive(Debug, Clone)]
pub struct Script {
    pub id: String,
    pub code: String,
    pub source: Option<String>,
}

impl Script {
    pub fn new(id: String, code: String) -> Self {
        Self {
            id,
            code,
            source: None,
        }
    }
    
    pub fn with_source(id: String, code: String, source: String) -> Self {
        Self {
            id,
            code,
            source: Some(source),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub value: JSValue,
    pub duration: std::time::Duration,
    pub memory_usage: u64,
}

pub struct ExecutionEngine {
    engine: Arc<dyn JSEngine>,
    resource_manager: Arc<ResourceManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    code_cache: Arc<RwLock<std::collections::HashMap<String, CompiledCode>>>,
}

#[derive(Debug, Clone)]
struct CompiledCode {
    code: String,
    compiled_at: Instant,
}

impl ExecutionEngine {
    pub fn new(engine: Box<dyn JSEngine>) -> Self {
        let resource_manager = Arc::new(ResourceManager::new());
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        
        Self {
            engine: Arc::from(engine),
            resource_manager,
            performance_monitor,
            code_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    pub async fn execute_script(&self, script: &Script) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        let cached_code = {
            let cache = self.code_cache.read().await;
            cache.get(&script.id).cloned()
        };
        
        let code_to_execute = if let Some(cached) = cached_code {
            cached.code
        } else {
            let code = script.code.clone();
            let mut cache = self.code_cache.write().await;
            cache.insert(script.id.clone(), CompiledCode {
                code: code.clone(),
                compiled_at: Instant::now(),
            });
            code
        };
        
        let result = self.engine.execute(&code_to_execute)?;
        
        let duration = start_time.elapsed();
        let memory_stats = self.engine.memory_usage();
        
        self.performance_monitor.record_execution(&script.id, duration, memory_stats.used);
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage: memory_stats.used,
        })
    }
    
    pub fn execute_script_sync(&self, script: &Script) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        let cached_code = {
            let cache = self.code_cache.blocking_read();
            cache.get(&script.id).cloned()
        };
        
        let code_to_execute = if let Some(cached) = cached_code {
            cached.code
        } else {
            let code = script.code.clone();
            let mut cache = self.code_cache.blocking_write();
            cache.insert(script.id.clone(), CompiledCode {
                code: code.clone(),
                compiled_at: Instant::now(),
            });
            code
        };
        
        let result = self.engine.execute(&code_to_execute)?;
        
        let duration = start_time.elapsed();
        let memory_stats = self.engine.memory_usage();
        
        self.performance_monitor.record_execution_sync(&script.id, duration, memory_stats.used);
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage: memory_stats.used,
        })
    }
    
    pub async fn call_function(&self, func: &str, args: Vec<JSValue>) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        let result = self.engine.call(func, args)?;
        
        let duration = start_time.elapsed();
        let memory_stats = self.engine.memory_usage();
        
        self.performance_monitor.record_execution(func, duration, memory_stats.used);
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage: memory_stats.used,
        })
    }
    
    pub fn call_function_sync(&self, func: &str, args: Vec<JSValue>) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        let result = self.engine.call(func, args)?;
        
        let duration = start_time.elapsed();
        let memory_stats = self.engine.memory_usage();
        
        self.performance_monitorOSCord_execution_sync(func, duration, memory_stats.used);
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage: memory_stats.used,
        })
    }
    
    pub async fn set_global(&self, name: &str, value: JSValue) -> Result<()> {
        self.engine.set_global(name, value)
    }
    
    pub fn set_global_sync(&self, name: &str, value: JSValue) -> Result<()> {
        self.engine.set_global(name, value)
    }
    
    pub async fn get_global(&self, name: &str) -> Result<JSValue> {
        self.engine.get_global(name)
    }
    
    pub fn get_global_sync(&self, name: &str) -> Result<JSValue> {
        self.engine.get_global(name)
    }
    
    pub fn memory_usage(&self) -> u64 {
        self.engine.memory_usage().used
    }
    
    pub fn gc(&self) -> Result<()> {
        self.engine.gc()
    }
    
    pub fn resource_manager(&self) -> Arc<ResourceManager> {
        Arc::clone(&self.resource_manager)
    }
    
    pub fn performance_monitor(&self) -> Arc<PerformanceMonitor> {
        Arc::clone(&self.performance_monitor)
    }
    
    pub async fn clear_cache(&self) {
        self.code_cache.write().await.clear();
    }
    
    pub fn clear_cache_sync(&self) {
        self.code_cache.blocking_write().clear();
    }
}