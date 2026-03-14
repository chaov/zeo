# zeo系统详细设计文档

**架构师**: Alex (首席架构师)  
**设计日期**: 2026年3月14日  
**版本**: v1.0  
**基于**: 系统架构设计文档

---

## 目录

1. [zeo执行引擎详细设计](#zeo执行引擎详细设计)
2. [LiteClaw框架详细设计](#liteclaw框架详细设计)
3. [集成层详细设计](#集成层详细设计)
4. [性能优化详细设计](#性能优化详细设计)
5. [移动端适配详细设计](#移动端适配详细设计)
6. [测试策略](#测试策略)
7. [开发指南](#开发指南)

---

## zeo执行引擎详细设计

### 1.1 项目结构

```
zeo/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── quickjs.rs
│   │   ├── jsc.rs
│   │   └── v8.rs
│   ├── runtime/
│   │   ├── mod.rs
│   │   ├── executor.rs
│   │   ├── scheduler.rs
│   │   └── async_runtime.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   ├── memory_pool.rs
│   │   ├── object_pool.rs
│   │   └── resource_manager.rs
│   ├── monitor/
│   │   ├── mod.rs
│   │   ├── performance.rs
│   │   ├── metrics.rs
│   │   └── reporter.rs
│   ├── compat/
│   │   ├── mod.rs
│   │   ├── openclaw.rs
│   │   ├── nodejs.rs
│   │   └── webapi.rs
│   ├── ffi/
│   │   ├── mod.rs
│   │   └── binding.rs
│   └── utils/
│       ├── mod.rs
│       ├── error.rs
│       └── config.rs
├── tests/
│   ├── integration/
│   └── unit/
└── benches/
```

### 1.2 核心模块设计

#### 1.2.1 引擎抽象层

**trait.rs**:

```rust
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::any::Any;
use std::sync::Arc;
use crate::utils::error::{Result, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JSValue {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSValue>),
    Object(Vec<(String, JSValue)>),
    Function(String),
}

impl JSValue {
    pub fn as_string(&self) -> Result<&str> {
        match self {
            JSValue::String(s) => Ok(s),
            _ => Err(Error::TypeError("Expected string".into())),
        }
    }
    
    pub fn as_number(&self) -> Result<f64> {
        match self {
            JSValue::Number(n) => Ok(*n),
            _ => Err(Error::TypeError("Expected number".into())),
        }
    }
    
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            JSValue::Bool(b) => Ok(*b),
            _ => Err(Error::TypeError("Expected bool".into())),
        }
    }
    
    pub fn as_array(&self) -> Result<&Vec<JSValue>> {
        match self {
            JSValue::Array(arr) => Ok(arr),
            _ => Err(Error::TypeError("Expected array".into())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            JSValue::Null => "null".to_string(),
            JSValue::Undefined => "undefined".to_string(),
            JSValue::Bool(b) => b.to_string(),
            JSValue::Number(n) => n.to_string(),
            JSValue::String(s) => s.clone(),
            JSValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            JSValue::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            JSValue::Function(name) => format!("[Function: {}]", name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub memory_limit: usize,
    pub stack_size: usize,
    pub enable_jit: bool,
    pub enable_profiler: bool,
    pub gc_threshold: usize,
    pub max_execution_time: Option<std::time::Duration>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            memory_limit: 128 * 1024 * 1024, // 128MB
            stack_size: 8 * 1024 * 1024,    // 8MB
            enable_jit: true,
            enable_profiler: false,
            gc_threshold: 16 * 1024 * 1024, // 16MB
            max_execution_time: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_used: usize,
    pub gc_count: usize,
    pub gc_time: std::time::Duration,
}

#[async_trait]
pub trait JSEngine: Send + Sync {
    fn new(config: EngineConfig) -> Result<Self> where Self: Sized;
    
    async fn execute(&self, code: &str) -> Result<JSValue>;
    async fn eval(&self, code: &str) -> Result<JSValue>;
    async fn call(&self, func: &str, args: Vec<JSValue>) -> Result<JSValue>;
    
    fn set_global(&self, name: &str, value: JSValue) -> Result<()>;
    fn get_global(&self, name: &str) -> Result<JSValue>;
    
    fn create_context(&self) -> Result<Box<dyn JSContext>>;
    
    fn memory_usage(&self) -> MemoryStats;
    async fn gc(&self) -> Result<()>;
    
    fn as_any(&self) -> &dyn Any;
}

pub trait JSContext: Send + Sync {
    async fn execute(&self, code: &str) -> Result<JSValue>;
    async fn eval(&self, code: &str) -> Result<JSValue>;
    
    fn set_variable(&self, name: &str, value: JSValue) -> Result<()>;
    fn get_variable(&self, name: &str) -> Result<JSValue>;
    
    fn memory_usage(&self) -> MemoryStats;
}

#[derive(Debug, Clone, Copy)]
pub enum EngineType {
    QuickJS,
    JavaScriptCore,
    V8,
}
```

#### 1.2.2 QuickJS引擎实现

**quickjs.rs**:

```rust
use crate::engine::trait::*;
use crate::utils::error::{Result, Error};
use quick_js::{Context, JsValue};
use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

pub struct QuickJSEngine {
    context: Arc<Context>,
    config: EngineConfig,
    stats: Arc<std::sync::RwLock<EngineStats>>,
}

#[derive(Debug, Default)]
struct EngineStats {
    gc_count: usize,
    gc_time: std::time::Duration,
    execution_count: usize,
    total_execution_time: std::time::Duration,
}

impl QuickJSEngine {
    fn convert_js_value(value: &JsValue) -> JSValue {
        match value {
            JsValue::Null => JSValue::Null,
            JsValue::Undefined => JSValue::Undefined,
            JsValue::Bool(b) => JSValue::Bool(*b),
            JsValue::Int(i) => JSValue::Number(*i as f64),
            JsValue::Float(f) => JSValue::Number(*f),
            JsValue::String(s) => JSValue::String(s.clone()),
            JsValue::Array(arr) => {
                JSValue::Array(arr.iter().map(Self::convert_js_value).collect())
            }
            JsValue::Object(obj) => {
                let mut result = Vec::new();
                for (key, value) in obj.iter() {
                    result.push((key.clone(), Self::convert_js_value(value)));
                }
                JSValue::Object(result)
            }
            _ => JSValue::Undefined,
        }
    }
    
    fn convert_to_js_value(value: &JSValue) -> Result<JsValue> {
        match value {
            JSValue::Null => Ok(JsValue::Null),
            JSValue::Undefined => Ok(JsValue::Undefined),
            JSValue::Bool(b) => Ok(JsValue::Bool(*b)),
            JSValue::Number(n) => Ok(JsValue::Float(*n)),
            JSValue::String(s) => Ok(JsValue::String(s.clone())),
            JSValue::Array(arr) => {
                let js_arr: Result<Vec<JsValue>> = arr.iter()
()
                    .map(Self::convert_to_js_value)
                    .collect();
                Ok(JsValue::Array(js_arr?))
            }
            JSValue::Object(obj) => {
                let mut js_obj = std::collections::HashMap::new();
                for (key, value) in obj.iter() {
                    js_obj.insert(key.clone(), Self::convert_to_js_value(value)?);
                }
                Ok(JsValue::Object(js_obj))
            }
            JSValue::Function(_) => Err(Error::NotImplemented("Function conversion".into())),
        }
    }
}

#[async_trait::async_trait]
impl JSEngine for QuickJSEngine {
    fn new(config: EngineConfig) -> Result<Self> {
        let context = Context::new()
            .map_err(|e| Error::EngineError(e.to_string()))?;
        
        Ok(Self {
            context: Arc::new(context),
            config,
            stats: Arc::new(std::sync::RwLock::new(EngineStats::default())),
        })
    }
    
    async fn execute(&self, code: &str) -> Result<JSValue> {
        let start = Instant::now();
        
        let result = self.context.eval(code)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        
        let duration = start.elapsed();
        {
            let mut stats = self.stats.write().unwrap();
            stats.execution_count += 1;
            stats.total_execution_time += duration;
        }
        
        Ok(Self::convert_js_value(&result))
    }
    
    async fn eval(&self, code: &str) -> Result<JSValue> {
        self.execute(code).await
    }
    
    async fn call(&self, func: &str, args: Vec<JSValue>) -> Result<JSValue> {
        let js_args: Result<Vec<JsValue>> = args.iter()
            .map(Self::convert_to_js_value)
            .collect();
        
        let result = self.context.call_function(func, &js_args?)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        
        Ok(Self::convert_js_value(&result))
    }
    
    fn set_global(&self, name: &str, value: JSValue) -> Result<()> {
        let js_value = Self::convert_to_js_value(&value)?;
        self.context.global().set(name, js_value)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        Ok(())
    }
    
    fn get_global(&self, name: &str) -> Result<JSValue> {
        let value = self.context.global().get(name)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        Ok(Self::convert_js_value(&value))
    }
    
    fn create_context(&self) -> Result<Box<dyn JSContext>> {
        let context = Context::new()
            .map_err(|e| Error::EngineError(e.to_string()))?;
        Ok(Box::new(QuickJSContext {
            context: Arc::new(context),
        }))
    }
    
    fn memory_usage(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.config.memory_limit,
            total_used: self.config.memory_limit / 2, // 估算
            gc_count: self.stats.read().unwrap().gc_count,
            gc_time: self.stats.read().unwrap().gc_time,
        }
    }
    
    async fn gc(&self) -> Result<()> {
        let start = Instant::now();
        
        self.context.memory_usage();
        
        let duration = start.elapsed();
        {
            let mut stats = self.stats.write().unwrap();
            stats.gc_count += 1;
            stats.gc_time += duration;
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct QuickJSContext {
    context: Arc<Context>,
}

#[async_trait::async_trait]
impl JSContext for QuickJSContext {
    async fn execute(&self, code: &str) -> Result<JSValue> {
        let result = self.context.eval(code)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        Ok(QuickJSEngine::convert_js_value(&result))
    }
    
    async fn eval(&self, code: &str) -> Result<JSValue> {
        self.execute(code).await
    }
    
    fn set_variable(&self, name: &str, value: JSValue) -> Result<()> {
        let js_value = QuickJSEngine::convert_to_js_value(&value)?;
        self.context.global().set(name, js_value)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        Ok(())
    }
    
    fn get_variable(&self, name: &str) -> Result<JSValue> {
        let value = self.context.global().get(name)
            .map_err(|e| Error::ExecutionError(e.to_string()))?;
        Ok(QuickJSEngine::convert_js_value(&value))
    }
    
    fn memory_usage(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: 16 * 1024 * 1024,
            total_used: 8 * 1024 * 1024,
            gc_count: 0,
            gc_time: std::time::Duration::default(),
        }
    }
}
```

#### 1.2.3 执行引擎核心

**executor.rs**:

```rust
use crate::engine::trait::*;
use crate::utils::error::{Result, Error};
use crate::resources::resource_manager::ResourceManager;
use crate::monitor::performance::PerformanceMonitor;
use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Script {
    pub id: String,
    pub code: String,
    pub source: Option<String>,
    pub is_module: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub value: JSValue,
    pub duration: std::time::Duration,
    pub memory_usage: MemoryStats,
}

#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    Sync,
    Async,
    Stream,
}

pub struct ExecutionEngine {
    engine: Arc<dyn JSEngine>,
    resource_manager: Arc<ResourceManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    code_cache: Arc<CodeCache>,
}

impl ExecutionEngine {
    pub fn new(
        engine: Arc<dyn JSEngine>,
        resource_manager: Arc<ResourceManager>,
        performance_monitor: Arc<PerformanceMonitor>,
    ) -> Self {
        Self {
            engine,
            resource_manager,
            performance_monitor,
            code_cache: Arc::new(CodeCache::new()),
        }
    }
    
    pub async fn execute_script(&self, script: &Script) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        let cached = self.code_cache.get_or_compile(script)?;
        let result = self.engine.execute(&cached).await?;
        
        let duration = start.elapsed();
        let memory_usage = self.engine.memory_usage();
        
        self.performance_monitor.record_execution(&script.id, duration);
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage,
        })
    }
    
    pub async fn execute_with_strategy(
        &self,
        script: &Script,
        strategy: ExecutionStrategy,
    ) -> Result<ExecutionResult> {
        match strategy {
            ExecutionStrategy::Sync => self.execute_script(script).await,
            ExecutionStrategy::Async => self.execute_async(script).await,
            ExecutionStrategy::Stream => self.execute_stream(script).await,
        }
    }
    
    async fn execute_async(&self, script: &Script) -> Result<ExecutionResult> {
        let engine = self.engine.clone();
        let script = script.clone();
        
        tokio::spawn(async move {
            let start = Instant::now();
            let result = engine.execute(&script.code).await?;
            let duration = start.elapsed();
            
            Ok(ExecutionResult {
                value: result,
                duration,
                memory_usage: engine.memory_usage(),
            })
        }).await?
    }
    
    async fn execute_stream(&self, script: &Script) -> Result<ExecutionResult> {
        let context = self.engine.create_context()?;
        let start = Instant::now();
        
        let result = context.execute(&script.code).await?;
        
        let duration = start.elapsed();
        
        Ok(ExecutionResult {
            value: result,
            duration,
            memory_usage: context.memory_usage(),
        })
    }
    
    pub fn get_engine(&self) -> Arc<dyn JSEngine> {
        self.engine.clone()
    }
}

pub struct CodeCache {
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

impl CodeCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    pub async fn get_or_compile(&self, script: &Script) -> Result<String> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&script.id) {
                return Ok(cached.clone());
            }
        }
        
        let compiled = self.compile(script)?;
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(script.id.clone(), compiled.clone());
        }
        
        Ok(compiled)
    }
    
    fn compile(&self, script: &Script) -> Result<String> {
        Ok(script.code.clone())
    }
    
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
}
```

#### 1.2.4 资源管理器

**resource_manager.rs**:

```rust
use crate::utils::error::{Result, Error};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
pub struct MemoryHandle {
    ptr: usize,
    size: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_used: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_objects: usize,
    pub max_execution_time: Option<std::time::Duration>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024, // 128MB
            max_objects: 10000,
            max_execution_time: None,
        }
    }
}

pub struct ResourceManager {
    memory_pool: Arc<MemoryPool>,
    limits: ResourceLimits,
    stats: Arc<ResourceStats>,
}

#[derive(Debug, Default)]
struct ResourceStats {
    allocated: AtomicUsize,
    used: AtomicUsize,
    objects: AtomicUsize,
}

impl ResourceManager {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            memory_pool: Arc::new(MemoryPool::new(limits.max_memory)),
            limits,
            stats: Arc::new(ResourceStats::default()),
        }
    }
    
    pub fn allocate(&self, size: usize) -> Result<MemoryHandle> {
        self.check_limits(size)?;
        
        let handle = self.memory_pool.allocate(size)?;
        
        self.stats.allocated.fetch_add(size, Ordering::Relaxed);
        self.stats.used.fetch_add(size, Ordering::Relaxed);
        self.stats.objects.fetch_add(1, Ordering::Relaxed);
        
        Ok(handle)
    }
    
    pub fn deallocate(&self, handle: MemoryHandle) {
        self.memory_pool.deallocate(handle);
        
        self.stats.used.fetch_sub(handle.size, Ordering::Relaxed);
        self.stats.objects.fetch_sub(1, Ordering::Relaxed);
    }
    
    fn check_limits(&self, size: usize) -> Result<()> {
        let current_used = self.stats.used.load(Ordering::Relaxed);
        
        if current_used + size > self.limits.max_memory {
            return Err(Error::ResourceLimitExceeded("Memory limit exceeded".into()));
        }
        
        let current_objects = self.stats.objects.load(Ordering::Relaxed);
        if current_objects >= self.limits.max_objects {
            return Err(Error::ResourceLimitExceeded("Object limit exceeded".into()));
        }
        
        Ok(())
    }
    
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.stats.allocated.load(Ordering::Relaxed),
            total_used: self.stats.used.load(Ordering::Relaxed),
            allocation_count: self.stats.objects.load(Ordering::Relaxed),
            deallocation_count: 0,
        }
    }
    
    pub fn set_limits(&mut self, limits: ResourceLimits) {
        self.limits = limits;
    }
}

pub struct MemoryPool {
    blocks: Vec<Vec<u8>>,
    block_size: usize,
    max_blocks: usize,
    free_blocks: Vec<usize>,
}

impl MemoryPool {
    pub fn new(max_memory: usize) -> Self {
        let block_size = 4096; // 4KB blocks
        let max_blocks = max_memory / block_size;
        
        Self {
            blocks: Vec::with_capacity(max_blocks),
            block_size,
            max_blocks,
            free_blocks: Vec::new(),
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Result<MemoryHandle> {
        let blocks_needed = (size + self.block_size - 1) / self.block_size;
        
        if blocks_needed > self.max_blocks {
            return Err(Error::OutOfMemory("Not enough memory".into()));
        }
        
        if self.free_blocks.len() >= blocks_needed {
            let start_block = self.free_blocks.drain(..blocks_needed).next().unwrap();
            let ptr = start_block * self.block_size;
            
            return Ok(MemoryHandle {
                ptr,
                size,
            });
        }
        
        let start_block = self.blocks.len();
        for _ in 0..blocks_needed {
            self.blocks.push(vec![0u8; self.block_size]);
        }
        
        let ptr = start_block * self.block_size;
        
        Ok(MemoryHandle {
            ptr,
            size,
        })
    }
    
    pub fn deallocate(&mut self, handle: MemoryHandle) {
        let start_block = handle.ptr / self.block_size;
        let blocks_used = (handle.size + self.block_size - 1) / self.block_size;
        
        for i in 0..blocks_used {
            self.free_blocks.push(start_block + i);
        }
    }
    
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.blocks.len() * self.block_size,
            total_used: (self.blocks.len() - self.free_blocks.len()) * self.block_size,
            allocation_count: self.blocks.len(),
            deallocation_count: self.free_blocks.len(),
        }
    }
}
```

#### 1.2.5 性能监控器

**performance.rs**:

```rust
use crate::utils::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    execution_times: HashMap<String, Vec<Duration>>,
    memory_usage: Vec<usize>,
    cpu_usage: Vec<f64>,
    gc_stats: Vec<GCStats>,
}

#[derive(Debug, Clone)]
pub struct GCStats {
    count: usize,
    total_time: Duration,
    last_time: Duration,
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub avg_execution_time: Duration,
    pub max_execution_time: Duration,
    pub min_execution_time: Duration,
    pub total_executions: usize,
    pub avg_memory_usage: usize,
    pub max_memory_usage: usize,
    pub avg_cpu_usage: f64,
    pub gc_count: usize,
    pub gc_total_time: Duration,
    pub bottlenecks: Vec<Bottleneck>,
}

#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub component: String,
    pub issue: String,
    pub severity: BottleneckSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PerformanceMonitor {
    metrics: Arc<tokio::sync::RwLock<PerformanceMetrics>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(PerformanceMetrics {
                execution_times: HashMap::new(),
                memory_usage: Vec::new(),
                cpu_usage: Vec::new(),
                gc_stats: Vec::new(),
            })),
            start_time: Instant::now(),
        }
    }
    
    pub async fn record_execution(&self, script_id: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.execution_times
            .entry(script_id.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub async fn record_memory_usage(&self, usage: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage.push(usage);
    }
    
    pub async fn record_cpu_usage(&self, usage: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cpu_usage.push(usage);
    }
    
    pub async fn record_gc(&self, stats: GCStats) {
        let mut metrics = self.metrics.write().await;
        metrics.gc_stats.push(stats);
    }
    
    pub async fn generate_report(&self) -> PerformanceReport {
        let metrics = self.metrics.read().await;
        
        let total_executions: usize = metrics.execution_times
            .values()
            .map(|times| times.len())
            .sum();
        
        let all_times: Vec<Duration> = metrics.execution_times
            .values()
            .flat_map(|times| times.iter().cloned())
            .collect();
        
        let avg_execution_time = if !all_times.is_empty() {
            let total: Duration = all_times.iter().sum();
            total / all_times.len() as u32
        } else {
            Duration::ZERO
        };
        
        let max_execution_time = all_times.iter().max().cloned().unwrap_or(Duration::ZERO);
        let min_execution_time = all_times.iter().min().cloned().unwrap_or(Duration::ZERO);
        
        let avg_memory_usage = if !metrics.memory_usage.is_empty() {
            let total: usize = metrics.memory_usage.iter().sum();
            total / metrics.memory_usage.len()
        } else {
            0
        };
        
        let max_memory_usage = metrics.memory_usage.iter().max().cloned().unwrap_or(0);
        
        let avg_cpu_usage = if !metrics.cpu_usage.is_empty() {
            let total: f64 = metrics.cpu_usage.iter().sum();
            total / metrics.cpu_usage.len() as f64
        } else {
            0.0
        };
        
        let gc_count: usize = metrics.gc_stats.iter().map(|s| s.count).sum();
        let gc_total_time: Duration = metrics.gc_stats.iter().map(|s| s.total_time).sum();
        
        let bottlenecks = self.analyze_bottlenecks(&metrics);
        
        PerformanceReport {
            avg_execution_time,
            max_execution_time,
            min_execution_time,
            total_executions,
            avg_memory_usage,
            max_memory_usage,
            avg_cpu_usage,
            gc_count,
            gc_total_time,
            bottlenecks,
        }
    }
    
    fn analyze_bottlenecks(&self, metrics: &PerformanceMetrics) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        if let Some(max_time) = metrics.execution_times.values().flat_map(|t| t.iter()).max() {
            if *max_time > Duration::from_secs(1) {
                bottlenecks.push(Bottleneck {
                    component: "Execution".to_string(),
                    issue: format!("Slow execution detected: {:?}", max_time),
                    severity: BottleneckSeverity::High,
                });
            }
        }
        
        if let Some(max_memory) = metrics.memory_usage.iter().max() {
            if *max_memory > 100 * 1024 * 1024 {
                bottlenecks.push(Bottleneck {
                    component: "Memory".to_string(),
                    issue: format!("High memory usage: {} MB", max_memory / 1024 / 1024),
                    severity: BottleneckSeverity::Medium,
                });
            }
        }
        
        if let Some(max_cpu) = metrics.cpu_usage.iter().max() {
            if *max_cpu > 80.0 {
                bottlenecks.push(Bottleneck {
                    component: "CPU".to_string(),
                    issue: format!("High CPU usage: {:.1}%", max_cpu),
                    severity: BottleneckSeverity::High,
                });
            }
        }
        
        bottlenecks
    }
    
    pub async fn clear(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.execution_times.clear();
        metrics.memory_usage.clear();
        metrics.cpu_usage.clear();
        metrics.gc_stats.clear();
    }
}
```

### 1.3 FFI绑定

**binding.rs**:

```rust
use crate::engine::trait::*;
use crate::runtime::executor executor::ExecutionEngine;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;
use std::sync::OnceLock;

static ENGINE: OnceLock<Arc<ExecutionEngine>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn zeo_init(config_json: *const c_char) -> c_int {
    let config_str = unsafe {
        if config_json.is_null() {
            return -1;
        }
        CStr::from_ptr(config_json).to_str()
    };
    
    if let Err(_) = config_str {
        return -1;
    }
    
    let config = EngineConfig::default();
    let engine = Arc::new(QuickJSEngine::new(config).unwrap());
    let execution_engine = Arc::new(ExecutionEngine::new(
        engine,
        Arc::new(ResourceManager::new(ResourceLimits::default())),
        Arc::new(PerformanceMonitor::new()),
    ));
    
    ENGINE.set(execution_engine).unwrap();
    
    0
}

#[no_mangle]
pub extern "C" fn zeo_execute(code: *const c_char) -> *mut c_char {
    if code.is_null() {
        return std::ptr::null_mut();
    }
    
    let code_str = unsafe { CStr::from_ptr(code).to_str() };
    if let Err(_) = code_str {
        return CString::new("Error: Invalid UTF-8").unwrap().into_raw();
    }
    
    let engine = ENGINE.get().unwrap();
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            let script = Script {
                id: uuid::Uuid::new_v4().to_string(),
                code: code_str.unwrap().to_string(),
                source: None,
                is_module: false,
            };
            engine.execute_script(&script).await
        });
    
    let result_str = match result {
        Ok(exec_result) => exec_result.value.to_string(),
        Err(e) => format!("Error: {}", e),
    };
    
    CString::new(result_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn zeo_call(func: *const c_char, args_json: *const c_char) -> *mut c_char {
    if func.is_null() || args_json.is_null() {
        return std::ptr::null_mut();
    }
    
    let func_str = unsafe { CStr::from_ptr(func).to_str() };
    let args_str = unsafe { CStr::from_ptr(args_json).to_str() };
    
    if let (Err(_), _) | (_, Err(_)) = (func_str, args_str) {
        return CString::new("Error: Invalid UTF-8").unwrap().into_raw();
    }
    
    let engine = ENGINE.get().unwrap();
    let js_engine = engine.get_engine();
    
    let args: Vec<JSValue> = match serde_json::from_str(args_str.unwrap()) {
        Ok(args) => args,
        Err(e) => {
            return CString::new(format!("Error: {}", e)).unwrap().into_raw();
        }
    };
    
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            js_engine.call(func_str.unwrap(), args).await
        });
    
    let result_str = match result {
        Ok(value) => value.to_string(),
        Err(e) => format!("Error: {}", e),
    };
    
    CString::new(result_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn zeo_set_global(name: *const c_char, value_json: *const c_char) -> c_int {
    if name.is_null() || value_json.is_null() {
        return -1;
    }
    
    let name_str = unsafe { CStr::from_ptr(name).to_str() };
    let value_str = unsafe { CStr::from_ptr(value_json).to_str() };
    
    if let (Err(_), _) | (_, Err(_)) = (name_str, value_str) {
        return -1;
    }
    
    let engine = ENGINE.get().unwrap();
    let js_engine = engine.get_engine();
    
    let value: JSValue = match serde_json::from_str(value_str.unwrap()) {
        Ok(value) => value,
        Err(_) => return -1,
    };
    
    match js_engine.set_global(name_str.unwrap(), value) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn zeo_get_global(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    
    let name_str = unsafe { CStr::from_ptr(name).to_str() };
    if let Err(_) = name_str {
        return CString::new("Error: Invalid UTF-8").unwrap().into_raw();
    }
    
    let engine = ENGINE.get().unwrap();
    let js_engine = engine.get_engine();
    
    let result = js_engine.get_global(name_str.unwrap());
    
    let result_str = match result {
        Ok(value) => value.to_string(),
        Err(e) => format!("Error: {}", e),
    };
    
    CString::new(result_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn zeo_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn zeo_gc() -> c_int {
    let engine = ENGINE.get().unwrap();
    let js_engine = engine.get_engine();
    
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            js_engine.gc().await
        });
    
    match result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn zeo_memory_stats() -> *mut c_char {
    let engine = ENGINE.get().unwrap();
    let js_engine = engine.get_engine();
    
    let stats = js_engine.memory_usage();
    
    let stats_json = serde_json::json!({
        "total_allocated": stats.total_allocated,
        "total_used": stats.total_used,
        "gc_count": stats.gc_count,
        "gc_time_ms": stats.gc_time.as_millis(),
    });
    
    CString::new(stats_json.to_string()).unwrap().into_raw()
}
```

---

## LiteClaw框架详细设计

### 2.1 项目结构

```
liteclaw/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── core/
│   │   ├── agent.ts
│   │   ├── tool.ts
│   │   ├── session.ts
│   │   ├── channel.ts
│   │   └── types.ts
│   ├── agents/
│   │   ├── base-agent.ts
│   │   ├── chat-agent.ts
│   │   ├── task-agent.ts
│   │   └── workflow-agent.ts
│   ├── tools/
│   │   ├── tool-registry.ts
│   │   ├── file-tool.ts
│   │   ├── http-tool.ts
│   │   ├── system-tool.ts
│   │   └── browser-tool.ts
│   ├── sessions/
│   │   ├── session-manager.ts
│   │   ├── memory-session.ts
│   │   └── persistent-session.ts
│   ├── channels/
│   │   ├── channel-adapter.ts
│   │   ├── cli-channel.ts
│   │   ├── http-channel.ts
│   │   └── websocket-channel.ts
│   ├── llm/
│   │   ├── client.ts
│   │   ├── providers.ts
│   │   └── types.ts
│   └── zeo/
│       ├── binding.ts
│       └── runtime.ts
├── tests/
│   ├── unit/
│   └── integration/
└── examples/
```

### 2.2 核心类型定义

**types.ts**:

```typescript
export interface Message {
  id: string;
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

export interface ToolCall {
  id: string;
  name: string;
  params: Record<string, any>;
}

export interface AgentResponse {
  id: string;
  content: string;
  toolCalls?: ToolCall[];
  metadata?: Record<string, any>;
}

export interface AgentConfig {
  id: string;
  name: string;
  description?: string;
  llm: LLMConfig;
  tools?: string[];
  maxHistory?: number;
  systemPrompt?: string;
}

export interface LLMConfig {
  provider: 'openai' | 'anthropic' | 'local';
  apiKey?: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
  streaming?: boolean;
}

export interface ToolConfig {
  name: string;
  description: string;
  parameters: Schema;
  permissions?: Permission[];
}

export interface Schema {
  type: 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null';
  properties?: Record<string, Schema>;
  items?: Schema;
  required?: string[];
  enum?: any[];
  description?: string;
}

export interface Permission {
  type: 'file' | 'network' | 'system' | 'process';
  action: 'read' | 'write' | 'execute' | 'control';
  resource?: string;
}

export interface SessionConfig {
  id: string;
  persist?: boolean;
  maxMessages?: number;
  ttl?: number;
}

export interface ChannelConfig {
  name: string;
  type: 'cli' | 'http' | 'websocket' | 'mobile';
  options?: Record<string, any>;
}
```

### 2.3 Agent核心实现

**base-agent.ts**:

```typescript
import { v4 as uuidv4 } from 'uuid';
import { Agent, Message, AgentResponse, AgentConfig, ToolCall } from '../core/types';
import { ToolRegistry } from '../tools/tool-registry';
import { SessionManager } from '../sessions/session-manager';
import { LLMClient } from '../llm/client';

export abstract class BaseAgent implements Agent {
  public id: string;
  public name: string;
  public config: AgentConfig;
  
  protected toolRegistry: ToolRegistry;
  protected sessionManager: SessionManager;
  protected llmClient: LLMClient;
  
  constructor(config: AgentConfig) {
    this.id = config.id;
    this.name = config.name;
    this.config = config;
    
    this.toolRegistry = new ToolRegistry();
    this.sessionManager = new SessionManager();
    this.llmClient = new LLMClient(config.llm);
    
    this.setupDefaultTools();
  }
  
  protected setupDefaultTools(): void {
    // Setup default tools
  }
  
  async execute(message: Message): Promise<AgentResponse> {
    const session = await this.sessionManager.getOrCreate(message.metadata?.sessionId || uuidv4());
    
    const context = await this.buildContext(session, message);
    
    const response = await this.llmClient.chat({
      messages: context,
      tools: this.toolRegistry.getToolSchemas(),
    });
    
    if (response.toolCalls && response.toolCalls.length > 0) {
      for (const toolCall of response.toolCalls) {
        const result = await this.executeToolCall(toolCall);
        response.content += `\nTool result: ${JSON.stringify(result)}`;
      }
    }
    
    session.addMessage(message);
    session.addMessage({
      id: uuidv4(),
      role: 'assistant',
      content: response.content,
      timestamp: new Date(),
    });
    
    return response;
  }
  
  async onMessage(message: Message): Promise<void> {
    await this.execute(message);
  }
  
  async onToolCall(tool: string, params: any): Promise<any> {
    return await this.toolRegistry.execute(tool, params, {
      agentId: this.id,
      timestamp: new Date(),
    });
  }
  
  async start(): Promise<void> {
    await this.llmClient.initialize();
  }
  
  async stop(): Promise<void> {
    await this.llmClient.close();
  }
  
  protected abstract buildContext(session: any, message: Message): Promise<Message[]>;
  
  private async executeToolCall(toolCall: ToolCall): Promise<any> {
    try {
      return await this.toolRegistry.execute(
        toolCall.name,
        toolCall.params,
        {
          agentId: this.id,
          timestamp: new Date(),
        }
      );
    } catch (error) {
      return {
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}
```

**chat-agent.ts**:

```typescript
import { BaseAgent } from './base-agent';
import { Message, AgentConfig } from '../core/types';

export class ChatAgent extends BaseAgent {
  constructor(config: AgentConfig) {
    super(config);
  }
  
  protected async buildContext(session: any, message: Message): Promise<Message[]> {
    const systemPrompt = this.config.systemPrompt || 
      'You are a helpful AI assistant.';
    
    const maxHistory = this.config.maxHistory || 10;
    const history = session.getMessages().slice(-maxHistory);
    
    return [
      {
        id: 'system',
        role: 'system',
        content: systemPrompt,
        timestamp: new Date(),
      },
      ...history,
      message,
    ];
  }
}
```

### 2.4 工具系统实现

**tool-registry.ts**:

```typescript
import { Tool, ToolConfig, ToolHandler, Permission, Schema } from '../core/types';

export interface ToolContext {
  agentId: string;
  timestamp: Date;
  sessionId?: string;
}

export type ToolHandler = (params: any, context: ToolContext) => Promise<any>;

export class ToolRegistry {
  private tools: Map<string, Tool> = new Map();
  
  register(tool: Tool): void {
    this.tools.set(tool.config.name, tool);
  }
  
  unregister(name: string): void {
    this.tools.delete(name);
  }
  
  get(name: string): Tool | undefined {
    return this.tools.get(name);
  }
  
  list(): Tool[] {
    return Array.from(this.tools.values());
  }
  
  async execute(name: string, params: any, context: ToolContext): Promise<any> {
    const tool = this.get(name);
    if (!tool) {
      throw new Error(`Tool not found: ${name}`);
    }
    
    await this.validateParams(tool.config, params);
    await this.checkPermissions(tool, context);
    
    return await tool.handler(params, context);
  }
  
  getToolSchemas(): any[] {
    return Array.from(this.tools.values()).map(tool => ({
      type: 'function',
      function: {
        name: tool.config.name,
        description: tool.config.description,
        parameters: tool.config.parameters,
      },
    }));
  }
  
  private async validateParams(config: ToolConfig, params: any): Promise<void> {
    if (config.parameters.required) {
      for (const required of config.parameters.required) {
        if (!(required in params)) {
          throw new Error(`Missing required parameter: ${required}`);
        }
      }
    }
  }
  
  private async checkPermissions(tool: Tool, context: ToolContext): Promise<void> {
    if (!tool.config.permissions || tool.config.permissions.length === 0) {
      return;
    }
    
    for (const permission of tool.config.permissions) {
      await this.checkPermission(permission, context);
    }
  }
  
  private async checkPermission(permission: Permission, context: ToolContext): Promise<void> {
    // Implement permission checking logic
  }
}

export class Tool {
  constructor(
    public config: ToolConfig,
    public handler: ToolHandler
  ) {}
}
```

**file-tool.ts**:

```typescript
import * as fs from 'fs/promises';
import { Tool, ToolContext } from './tool-registry';
import { ToolConfig, Permission } from '../core/types';

export class FileTool extends Tool {
  constructor() {
    const config: ToolConfig = {
      name: 'file',
      description: 'File operations: read, write, delete, list',
      parameters: {
        type: 'object',
        properties: {
          action: {
            type: 'string',
            enum: ['read', 'write', 'delete', 'list'],
            description: 'The action to perform',
          },
          path: {
            type: 'string',
            description: 'The file path',
          },
          content: {
            type: 'string',
            description: 'The content to write (for write action)',
          },
        },
        required: ['action', 'path'],
      },
      permissions: [
        { type: 'file', action: 'read' },
        { type: 'file', action: 'write' },
      ],
    };
    
    super(config, FileTool.handler);
  }
  
  private static async handler(params: any, context: ToolContext): Promise<any> {
    const { action, path, content } = params;
    
    switch (action) {
      case 'read':
        return await fs.readFile(path, 'utf-8');
      
      case 'write':
        await fs.writeFile(path, content, 'utf-8');
        return { success: true, path };
      
      case 'delete':
        await fs.unlink(path);
        return { success: true, path };
      
      case 'list':
        const files = await fs.readdir(path, { withFileTypes: true });
        return files.map(file => ({
          name: file.name,
          isDirectory: file.isDirectory(),
        }));
      
      default:
        throw new Error(`Unknown action: ${action}`);
    }
  }
}
```

### 2.5 会话管理实现

**session-manager.ts**:

```typescript
import { Session, SessionConfig } from '../core/types';
import { MemorySession } from './memory-session';
import { PersistentSession } from './persistent-session';

export class SessionManager {
  private sessions: Map<string, Session> = new Map();
  
  async create(config: SessionConfig): Promise<Session> {
    let session: Session;
    
    if (config.persist) {
      session = new PersistentSession(config);
    } else {
      session = new MemorySession(config);
    }
    
    await session.initialize();
    this.sessions.set(config.id, session);
    
    return session;
  }
  
  async get(id: string): Promise<Session | undefined> {
    return this.sessions.get(id);
  }
  
  async getOrCreate(id: string): Promise<Session> {
    let session = await this.get(id);
    if (!session) {
      session = await this.create({ id, persist: false });
    }
    return session;
  }
  
  async delete(id: string): Promise<void> {
    const session = this.sessions.get(id);
    if (session) {
      await session.close();
      this.sessions.delete(id);
    }
  }
  
  async list(): Promise<Session[]> {
    return Array.from(this.sessions.values());
  }
  
  async clear(): Promise<void> {
    for (const session of this.sessions.values()) {
      await session.close();
    }
    this.sessions.clear();
  }
}
```

**memory-session.ts**:

```typescript
import { Session, SessionConfig, Message } from '../core/types';

export class MemorySession implements Session {
  public id: string;
  public createdAt: Date;
  public updatedAt: Date;
  public messages: Message[] = [];
  public metadata: Record<string, any> = {};
  
  private config: SessionConfig;
  
  constructor(config: SessionConfig) {
    this.id = config.id;
    this.config = config;
    this.createdAt = new Date();
    this.updatedAt = new Date();
  }
  
  async initialize(): Promise<void> {
    // Initialize session
  }
  
  async close(): Promise<void> {
    // Cleanup session
  }
  
  addMessage(message: Message): void {
    this.messages.push(message);
    this.updatedAt = new Date();
    
    if (this.config.maxMessages && this.messages.length > this.config.maxMessages) {
      this.messages = this.messages.slice(-this.config.maxMessages);
    }
  }
  
  getMessages(): Message[] {
    return this.messages;
  }
  
  setMetadata(key: string, value: any): void {
    this.metadata[key] = value;
    this.updatedAt = new Date();
  }
  
  getMetadata(key: string): any {
    return this.metadata[key];
  }
}
```

### 2.6 zeo绑定实现

**binding.ts**:

```typescript
declare module 'zeo' {
  export function execute(code: string): string;
  export function call(func: string, args: string): string;
  export function setGlobal(name: string, value: string): number;
  export function getGlobal(name: string): string;
  export function gc(): number;
  export function memoryStats(): string;
  export function free(ptr: number): void;
}

import * as zeo from 'zeo';

export class ZeoBinding {
  static execute(code: string): any {
    const result = zeo.execute(code);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }
  
  static call(func: string, args: any[]): any {
    const argsJson = JSON.stringify(args);
    const result = zeo.call(func, argsJson);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }
  
  static setGlobal(name: string, value: any): void {
    const valueJson = JSON.stringify(value);
    const result = zeo.setGlobal(name, valueJson);
    if (result !== 0) {
      throw new Error(`Failed to set global variable: ${name}`);
    }
  }
  
  static getGlobal(name: string): any {
    const result = zeo.getGlobal(name);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }
  
  static gc(): void {
    const result = zeo.gc();
    if (result !== 0) {
      throw new Error('Failed to perform garbage collection');
    }
  }
  
  static memoryStats(): any {
    const result = zeo.memoryStats();
    return JSON.parse(result);
  }
}
```

**runtime.ts**:

```typescript
import { ZeoBinding } from './binding';

export class ZeoRuntime {
  private binding: typeof ZeoBinding;
  
  constructor() {
    this.binding = ZeoBinding;
  }
  
  execute(code: string): any {
    return this.binding.execute(code);
  }
  
  call(func: string, args: any[]): any {
    return this.binding.call(func, args);
  }
  
  setGlobal(name: string, value: any): void {
    this.binding.setGlobal(name, value);
  }
  
  getGlobal(name: string): any {
    return this.binding.getGlobal(name);
  }
  
  gc(): void {
    this.binding.gc();
  }
  
  memoryStats(): any {
    return this.binding.memoryStats();
  }
  
  async executeAsync(code: string): Promise<any> {
    return new Promise((resolve, reject) => {
      try {
        const result = this.execute(code);
        resolve(result);
      } catch (error) {
        reject(error);
      }
    });
  }
}
```

---

## 集成层详细设计

### 3.1 OpenClaw协议兼容

**openclaw.rs**:

```rust
use serde::{Deserialize, Serialize};
use crate::utils::error::{Result, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenClawMessage {
    #[serde(rename = "connect")]
    Connect { id: String, token: Option<String> },
    
    #[serde(rename = "req")]
    Request { id: String, method: String, params: serde_json::Value },
    
    #[serde(rename = "res")]
    Response { id: String, ok: bool, payload: Option<serde_json::Value>, error: Option<String> },
    
    #[serde(rename = "event")]
    Event { event: String, payload: serde_json::Value },
}

pub struct OpenClawProtocol {
    sessions: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Session>>>,
}

#[derive(Debug, Clone)]
struct Session {
    id: String,
    connected: bool,
    metadata: std::collections::HashMap<String, String>,
}

impl OpenClawProtocol {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    pub async fn handle_message(&self, message: &str) -> Result<String> {
        let msg: OpenClawMessage = serde_json::from_str(message)
            .map_err(|e| Error::ParseError(e.to_string()))?;
        
        let response = match msg {
            OpenClawMessage::Connect { id, token } => {
                self.handle_connect(id, token).await?
            }
            OpenClawMessage::Request { id, method, params } => {
                self.handle_request(id, method, params).await?
            }
            OpenClawMessage::Event { event, payload } => {
                self.handle_event(event, payload).await?
            }
            OpenClawMessage::Response { .. } => {
                return Err(Error::InvalidMessage("Unexpected response".into()));
            }
        };
        
        serde_json::to_string(&response)
            .map_err(|e| Error::SerializeError(e.to_string()))
    }
    
    async fn handle_connect(&self, id: String, _token: Option<String>) -> Result<OpenClawMessage> {
        let session = Session {
            id: id.clone(),
            connected: true,
            metadata: std::collections::HashMap::new(),
        };
        
        self.sessions.write().await.insert(id.clone(), session);
        
        Ok(OpenClawMessage::Response {
            id: "connect".to_string(),
            ok: true,
            payload: Some(serde_json::json!({ "sessionId": id })),
            error: None,
        })
    }
    
    async fn handle_request(&self, id: String, method: String, params: serde_json::Value) -> Result<OpenClawMessage> {
        let session_id = params["session_id"].as_str()
            .ok_or_else(|| Error::InvalidMessage("Missing session_id".into()))?;
        
        let sessions = self.sessions.read().await;
        let _session = sessions.get(session_id)
            .ok_or_else(|| Error::SessionNotFound(session_id.to_string()))?;
        
        let result = self.execute_method(&method, params).await?;
        
        Ok(OpenClawMessage::Response {
            id,
            ok: true,
            payload: Some(result),
            error: None,
        })
    }
    
    async fn handle_event(&self, event: String, payload: serde_json::Value) -> Result<OpenClawMessage> {
        Ok(OpenClawMessage::Response {
            id: event,
            ok: true,
            payload: Some(payload),
            error: None,
        })
    }
    
    async fn execute_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        match method {
            "agent.execute" => self.agent_execute(params).await,
            "tool.call" => self.tool_call(params).await,
            "session.get" => self.session_get(params).await,
            _ => Err(Error::MethodNotFound(method.to_string())),
        }
    }
    
    async fn agent_execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let message = params["message"]
            .as_str()
            .ok_or_else(|| Error::InvalidMessage("Missing message".into()))?;
        
        Ok(serde_json::json!({
            "response": format!("Processed: {}", message),
        }))
    }
    
    async fn tool_call(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let tool = params["tool"]
            .as_str()
            .ok_or_else(|| Error::InvalidMessage("Missing tool".into()))?;
        
        Ok(serde_json::json!({
            "result": format!("Tool {} executed", tool),
        }))
    }
    
    async fn session_get(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let session_id = params["session_id"]
            .as_str()
            .ok_or_else(|| Error::InvalidMessage("Missing session_id".into()))?;
        
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| Error::SessionNotFound(session_id.to_string()))?;
        
        Ok(serde_json::json!({
            "sessionId": session.id,
            "connected": session.connected,
        }))
    }
}
```

### 3.2 Node.js API兼容

**nodejs.rs**:

```rust
use crate::engine::trait::*;
use crate::utils::error::{Result, Error};

pub struct NodeCompatLayer {
    engine: Arc<dyn JSEngine>,
}

impl NodeCompatLayer {
    pub fn new(engine: Arc<dyn JSEngine>) -> Self {
        Self { engine }
    }
    
    pub fn setup_node_globals(&self) -> Result<()> {
        self.setup_process()?;
        self.setup_console()?;
        self.setup_buffer()?;
        self.setup_require()?;
        self.setup_timers()?;
        Ok(())
    }
    
    fn setup_process(&self) -> Result<()> {
        let process = r#"
            global.process = {
                version: '1.0.0',
                platform: 'linux',
                arch: 'x64',
                env: {},
                argv: [],
                exit: function(code) {
                    // Handle exit
                },
            };
        "#;
        
        self.engine.eval(process)?;
        Ok(())
    }
    
    fn setup_console(&self) -> Result<()> {
        let console = r#"
            global.console = {
                log: function(...args) {
                    print(args.join(' '));
                },
                error: function(...args) {
                    print('ERROR: ' + args.join(' '));
                },
                warn: function(...args) {
                    print('WARN: ' + args.join(' '));
                },
                info: function(...args) {
                    print('INFO: ' + args.join(' '));
                },
            };
        "#;
        
        self.engine.eval(console)?;
        Ok(())
    }
    
    fn setup_buffer(&self) -> Result<()> {
        let buffer = r#"
            global.Buffer = {
                from: function(data) {
                    if (typeof data === 'string') {
                        return new TextEncoder().encode(data);
                    }
                    return data;
                },
                isBuffer: function(obj) {
                    return obj instanceof Uint8Array;
                },
            };
        "#;
        
        self.engine.eval(buffer)?;
        Ok(())
    }
    
    fn setup_require(&self) -> Result<()> {
        let require = r#"
            global.require = function(module) {
                // Simple module resolution
                if (module === 'fs') {
                    return {
                        readFileSync: function(path) {
                            return 'File content';
                        },
                        writeFileSync: function(path, content) {
                            // Write file
                        },
                    };
                }
                throw new Error('Module not found: ' + module);
            };
        "#;
        
        self.engine.eval(require)?;
        Ok(())
    }
    
    fn setup_timers(&self) -> Result<()> {
        let timers = r#"
            global.setTimeout = function(callback, delay) {
                // Simple setTimeout implementation
                callback();
                return 1;
            };
            
            global.setInterval = function(callback, delay) {
                // Simple setInterval implementation
                callback();
                return 1;
            };
            
            global.clearTimeout = function(id) {
                // Clear timeout
            };
            
            global.clearInterval = function(id) {
                // Clear interval
            };
        "#;
        
        self.engine.eval(timers)?;
        Ok(())
    }
}
```

---

## 性能优化详细设计

### 4.1 JIT优化

```rust
pub struct JITOptimizer {
    hot_functions: Arc<tokio::sync::RwLock<std::collections::HashMap<String, FunctionStats>>>,
    compilation_threshold: usize,
}

#[derive(Debug, Clone)]
struct FunctionStats {
    call_count: usize,
    total_time: std::time::Duration,
    is_compiled: bool,
}

impl JITOptimizer {
    pub fn new(threshold: usize) -> Self {
        Self {
            hot_functions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            compilation_threshold: threshold,
        }
    }
    
    pub async fn record_call(&self, func: &str, duration: std::time::Duration) {
        let mut stats = self.hot_functions.write().await;
        let func_stats = stats.entry(func.to_string()).or_insert_with(|| FunctionStats {
            call_count: 0,
            total_time: std::time::Duration::ZERO,
            is_compiled: false,
        });
        
        func_stats.call_count += 1;
        func_stats.total_time += duration;
        
        if func_stats.call_count >= self.compilation_threshold && !func_stats.is_compiled {
            self.compile_function(func).await;
            func_stats.is_compiled = true;
        }
    }
    
    async fn compile_function(&self, func: &str) {
        // JIT compilation logic
    }
}
```

### 4.2 内存优化

```rust
pub struct MemoryOptimizer {
    pools: Vec<MemoryPool>,
    strategy: MemoryStrategy,
}

#[derive(Debug, Clone)]
pub enum MemoryStrategy {
    Conservative,
    Balanced,
    Aggressive,
}

impl MemoryOptimizer {
    pub fn new(strategy: MemoryStrategy) -> Self {
        Self {
            pools: Vec::new(),
            strategy,
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Result<MemoryHandle> {
        for pool in &mut self.pools {
            if pool.can_allocate(size) {
                return pool.allocate(size);
            }
        }
        
        let new_pool = MemoryPool::new(size * 2);
        self.pools.push(new_pool);
        self.pools.last_mut().unwrap().allocate(size)
    }
    
    pub fn optimize(&mut self) {
        match self.strategy {
            MemoryStrategy::Conservative => self.conservative_gc(),
            MemoryStrategy::Balanced => self.balanced_gc(),
            MemoryStrategy::Aggressive => self.aggressive_gc(),
        }
    }
    
    fn conservative_gc(&mut self) {
        // Conservative GC
    }
    
    fn balanced_gc(&mut self) {
        // Balanced GC
    }
    
    fn aggressive_gc(&mut self) {
        // Aggressive GC
    }
}
```

---

## 移动端适配详细设计

### 5.1 移动端配置

```rust
#[cfg(feature = "mobile")]
pub mod mobile {
    use super::*;
    
    pub fn create_mobile_engine() -> Result<Box<dyn JSEngine>> {
        let config = EngineConfig {
            memory_limit: 16 * 1024 * 1024, // 16MB
            stack_size: 2 * 1024 * 1024,     // 2MB
            enable_jit: false,
            enable_profiler: false,
            gc_threshold: 4 * 1024 * 1024,  // 4MB
            max_execution_time: Some(std::time::Duration::from_secs(5)),
        };
        
        Ok(Box::new(QuickJSEngine::new(config)?))
    }
    
    pub fn create_mobile_runtime() -> Result<ExecutionEngine> {
        let engine = Arc::new(create_mobile_engine()?);
        let resource_manager = Arc::new(ResourceManager::new(ResourceLimits {
            max_memory: 16 * 1024 * 1024,
            max_objects: 1000,
            max_execution_time: Some(std::time::Duration::from_secs(5)),
        }));
        
        Ok(ExecutionEngine::new(
            engine,
            resource_manager,
            Arc::new(PerformanceMonitor::new()),
        ))
    }
}
```

### 5.2 电池优化

```rust
pub struct BatteryAwareScheduler {
    battery_level: Arc<std::sync::atomic::AtomicU8>,
    is_charging: Arc<std::sync::atomic::AtomicBool>,
    low_power_mode: Arc<std::sync::atomic::AtomicBool>,
}

impl BatteryAwareScheduler {
    pub fn new() -> Self {
        Self {
            battery_level: Arc::new(std::sync::atomic::AtomicU8::new(100)),
            is_charging: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            low_power_mode: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub fn update_battery_status(&self, level: u8, charging: bool) {
        self.battery_level.store(level, std::sync::atomic::Ordering::Relaxed);
        self.is_charging.store(charging, std::sync::atomic::Ordering::Relaxed);
        
        let low_power = level < 20 && !charging;
        self.low_power_mode.store(low_power, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn should_execute(&self) -> bool {
        if self.low_power_mode.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }
        true
    }
    
    pub fn get_execution_delay(&self) -> std::time::Duration {
        let level = self.battery_level.load(std::sync::atomic::Ordering::Relaxed);
        
        if level < 20 {
            std::time::Duration::from_secs(10)
        } else if level < 50 {
            std::time::Duration::from_secs(5)
        } else {
            std::time::Duration::ZERO
        }
    }
}
```

---

## 测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_execution() {
        let engine = QuickJSEngine::new(EngineConfig::default()).unwrap();
        let result = engine.execute("1 + 1").await.unwrap();
        
        assert_eq!(result, JSValue::Number(2.0));
    }
    
    #[tokio::test]
    async fn test_engine_call() {
        let engine = QuickJSEngine::new(EngineConfig::default()).unwrap();
        engine.eval("function add(a, b) { return a + b; }").await.unwrap();
        
        let result = engine.call("add", vec![
            JSValue::Number(1.0),
            JSValue::Number(2.0),
        ]).await.unwrap();
        
        assert_eq!(result, JSValue::Number(3.0));
    }
    
    #[tokio::test]
    async fn test_memory_pool() {
        let mut pool = MemoryPool::new(1024 * 1024);
        
        let handle1 = pool.allocate(1024).unwrap();
        let handle2 = pool.allocate(2048).unwrap();
        
        assert_eq!(handle1.ptr, 0);
        assert_eq!(handle2.ptr, 4096);
        
        pool.deallocate(handle1);
        pool.deallocate(handle2);
    }
}
```

### 6.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agent_execution() {
        let engine = Arc::new(QuickJSEngine::new(EngineConfig::default()).unwrap());
        let execution_engine = Arc::new(ExecutionEngine::new(
            engine,
            Arc::new(ResourceManager::new(ResourceLimits::default())),
            Arc::new(PerformanceMonitor::new()),
        ));
        
        let script = Script {
            id: "test".to_string(),
            code: "function hello() { return 'Hello, World!'; } hello();".to_string(),
            source: None,
            is_module: false,
        };
        
        let result = execution_engine.execute_script(&script).await.unwrap();
        
        assert_eq!(result.value, JSValue::String("Hello, World!".to_string()));
    }
}
```

---

## 开发指南

### 7.1 构建指南

```bash
# 构建zeo
cargo build --release

# 构建LiteClaw
cd liteclaw
npm install
npm run build

# 构建移动端
cargo build --release --features mobile
```

### 7.2 测试指南

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration

# 运行性能测试
cargo bench

# 运行LiteClaw测试
cd liteclaw
npm test
```

### 7.3 部署指南

```bash
# 部署到生产环境
cargo build --release
./target/release/zeo

# Docker部署
docker build -t zeo:latest .
docker run -p 3000:3000 zeo:latest
```

---

## 总结

本设计文档提供了zeo系统的详细设计，包括：

1. **zeo执行引擎**：基于Rust的高性能执行引擎
2. **LiteClaw框架**：基于TypeScript的AI Agent框架
3. **集成层**：OpenClaw和Node.js兼容层
4. **性能优化**：JIT、内存、执行效率优化
5. **移动端适配**：移动端优化和电池管理
6. **测试策略**：单元测试和集成测试
7. **开发指南**：构建、测试、部署指南

该设计旨在实现：
- 性能优于Bun 50%以上
- 移动端友好
- 引擎可切换
- 完整兼容性

---

**文档结束**

*本设计文档基于系统架构设计，具体实施时需要进一步的验证和调整。*