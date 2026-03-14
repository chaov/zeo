use std::sync::{Arc, Mutex, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub ptr: *mut u8,
    pub size: usize,
    pub allocated_at: Instant,
}

unsafe impl Send for MemoryBlock {}
unsafe impl Sync for MemoryBlock {}

pub struct MemoryPool {
    pools: Vec<VecDeque<MemoryBlock>>,
    block_sizes: Vec<usize>,
    max_blocks_per_pool: usize,
    total_allocated: Arc<AtomicU64>,
}

impl MemoryPool {
    pub fn new(base_size: usize, max_blocks: usize) -> Self {
        let mut pools = Vec::new();
        let mut block_sizes = Vec::new();
        
        for i in 0..8 {
            let size = base_size * (2usize.pow(i as u32));
            pools.push(VecDeque::with_capacity(max_blocks));
            block_sizes.push(size);
        }
        
        Self {
            pools,
            block_sizes,
            max_blocks_per_pool: max_blocks,
            total_allocated: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<MemoryBlock> {
        let pool_index = self.find_pool_index(size);
        
        if pool_index >= self.pools.len() {
            return None;
        }
        
        if let Some(block) = self.pools[pool_index].pop_front() {
            return Some(block);
        }
        
        let actual_size = self.block_sizes[pool_index];
        let ptr = unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(actual_size, 8)) };
        
        if ptr.is_null() {
            return None;
        }
        
        self.total_allocated.fetch_add(actual_size as u64, Ordering::Relaxed);
        
        Some(MemoryBlock {
            ptr,
            size: actual_size,
            allocated_at: Instant::now(),
        })
    }
    
    pub fn deallocate(&mut self, block: MemoryBlock) {
        let pool_index = self.find_pool_index(block.size);
        
        if pool_index < self.pools.len() && self.pools[pool_index].len() < self.max_blocks_per_pool {
            self.pools[pool_index].push_back(block);
        } else {
            unsafe {
                std::alloc::dealloc(
                    block.ptr,
                    std::alloc::Layout::from_size_align_unchecked(block.size, 8)
                );
            }
            self.total_allocated.fetch_sub(block.size as u64, Ordering::Relaxed);
        }
    }
    
    fn find_pool_index(&self, size: usize) -> usize {
        for (i, &block_size) in self.block_sizes.iter().enumerate() {
            if size <= block_size {
                return i;
            }
        }
        self.block_sizes.len()
    }
    
    pub fn stats(&self) -> MemoryStats {
        let total_capacity = self.block_sizes.iter().sum::<usize>();
        let used = self.total_allocated.load(Ordering::Relaxed);
        
        MemoryStats {
            total: total_capacity as u64,
            used,
            peak: used,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub peak: u64,
}

#[derive(Debug, Clone)]
pub struct ObjectLimits {
    pub max_memory: usize,
    pub max_objects: usize,
    pub max_execution_time: std::time::Duration,
}

impl Default for ObjectLimits {
    fn default() -> Self {
        Self {
            max_memory: 16 * 1024 * 1024, // 16MB
            max_objects: 10000,
            max_execution_time: std::time::Duration::from_secs(30),
        }
    }
}

pub struct GCScheduler {
    last_gc: Arc<Mutex<Instant>>,
    gc_interval: std::time::Duration,
    memory_threshold: f64,
    is_gc_running: Arc<AtomicBool>,
}

impl GCScheduler {
    pub fn new(interval: std::time::Duration, threshold: f64) -> Self {
        Self {
            last_gc: Arc::new(Mutex::new(Instant::now())),
            gc_interval: interval,
            memory_threshold: threshold,
            is_gc_running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn should_gc(&self, memory_stats: &MemoryStats) -> bool {
        if self.is_gc_running.load(Ordering::Relaxed) {
            return false;
        }
        
        let last_gc = self.last_gc.lock().unwrap();
        let time_since_gc = last_gc.elapsed();
        
        if time_since_gc < self.gc_interval {
            return false;
        }
        
        if memory_stats.total > 0 {
            let usage_ratio = memory_stats.used as f64 / memory_stats.total as f64;
            usage_ratio >= self.memory_threshold
        } else {
            false
        }
    }
    
    pub fn mark_gc_start(&self) {
        self.is_gc_running.store(true, Ordering::Relaxed);
    }
    
    pub fn mark_gc_complete(&self) {
        self.is_gc_running.store(false, Ordering::Relaxed);
        *self.last_gc.lock().unwrap() = Instant::now();
    }
}

pub struct LeakDetector {
    allocations: Arc<Mutex<std::collections::HashMap<usize, AllocationInfo>>>,
    next_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    allocated_at: Instant,
    backtrace: Option<Vec<String>>,
}

impl LeakDetector {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn track_allocation(&self, size: usize) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) as usize;
        
        let info = AllocationInfo {
            size,
            allocated_at: Instant::now(),
            backtrace: None,
        };
        
        self.allocations.lock().unwrap().insert(id, info);
        id
    }
    
    pub fn track_deallocation(&self, id: usize) {
        self.allocations.lock().unwrap().remove(&id);
    }
    
    pub fn detect_leaks(&self, max_age: std::time::Duration) -> Vec<LeakInfo> {
        let allocations = self.alloc.allocations.lock().unwrap();
        let mut leaks = Vec::new();
        
        for (id, info) in allocations.iter() {
            if info.allocated_at.elapsed() > max_age {
                leaks.push(LeakInfo {
                    id: *id,
                    size: info.size,
                    age: info.allocated_at.elapsed(),
                });
            }
        }
        
        leaks
    }
}

#[derive(Debug, Clone)]
pub struct LeakInfo {
    pub id: usize,
    pub size: usize,
    pub age: std::time::Duration,
}

pub struct ResourceManager {
    memory_pool: Arc<Mutex<MemoryPool>>,
    resource_limits: ObjectLimits,
    gc_scheduler: Arc<GCScheduler>,
    leak_detector: Arc<LeakDetector>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let memory_pool = MemoryPool::new(1024, 100);
        let gc_scheduler = GCScheduler::new(
            std::time::Duration::from_secs(5),
            0.8,
        );
        
        Self {
            memory_pool: Arc::new(Mutex::new(memory_pool)),
            resource_limits: ObjectLimits::default(),
            gc_scheduler: Arc::new(gc_scheduler),
            leak_detector: Arc::new(LeakDetector::new()),
        }
    }
    
    pub fn allocate(&self, size: usize) -> Result<MemoryHandle, ResourceError> {
        if size > self.resource_limits.max_memory {
            return Err(ResourceError::MemoryLimitExceeded(size));
        }
        
        let mut pool = self.memory_pool.lock().unwrap();
        
        if let Some(block) = pool.allocate(size) {
            let id = self.leak_detector.track_allocation(size);
            Ok(MemoryHandle {
                id,
                block,
                manager: self.clone(),
            })
        } else {
            Err(ResourceError::OutOfMemory)
        }
    }
    
    pub fn set_limits(&mut self, limits: ObjectLimits) {
        self.resource_limits = limits;
    }
    
    pub fn memory_stats(&self) -> MemoryStats {
        self.memory_pool.lock().unwrap().stats()
    }
    
    pub fn should_gc(&self) -> bool {
        let stats = self.memory_stats();
        self.gc_scheduler.should_gc(&stats)
    }
    
    pub fn trigger_gc(&self) {
        self.gc_scheduler.mark_gc_start();
        self.gc_scheduler.mark_gc_complete();
    }
    
    pub fn detect_leaks(&self, max_age: std::time::Duration) -> Vec<LeakInfo> {
        self.leak_detector.detect_leaks(max_age)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryHandle {
    id: usize,
    block: MemoryBlock,
    manager: ResourceManager,
}

impl MemoryHandle {
    pub fn ptr(&self) -> *mut u8 {
        self.block.ptr
    }
    
    pub fn size(&self) -> usize {
        self.block.size
    }
}

impl Drop for MemoryHandle {
    fn drop(&mut self) {
        self.manager.leak_detector.track_deallocation(self.id);
        let mut pool = self.manager.memory_pool.lock().unwrap();
        pool.deallocate(std::mem::take(&mut self.block));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceError {
    #[error("Memory limit exceeded: requested {0} bytes")]
    MemoryLimitExceeded(usize),
    
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Resource limit exceeded")]
    LimitExceeded,
}

pub type Result<T> = std::result::Result<T, ResourceError>;