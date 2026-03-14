use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{info, error};

mod engine;
mod core;
mod resource;
mod monitor;
mod integration;

use engine::{JSEngine, EngineType, EngineConfig, EngineFactory};
use core::ExecutionEngine;
use resource::ResourceManager;
use monitor::PerformanceMonitor;

static ENGINE: OnceLock<Arc<Mutex<Option<ExecutionEngine>>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn zeo_init(engine_type: u32) -> i32 {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let engine_type = match engine_type {
        0 => EngineType::QuickJS,
        1 => EngineType::JavaScriptCore,
        2 => EngineType::V8,
        _ => EngineType::QuickJS,
    };

    info!("Initializing zeo with engine: {:?}", engine_type);

    let config = EngineConfig {
        memory_limit: 16 * 1024 * 1024, // 16MB default
        enable_jit: true,
        enable_profiler: false,
    };

    match EngineFactory::create(engine_type, config) {
        Ok(engine) => {
            let execution_engine = ExecutionEngine::new(engine);
            ENGINE.get_or_init(|| Arc::new(Mutex::new(Some(execution_engine))));
            0
        }
        Err(e) => {
            error!("Failed to initialize engine: {}", e);
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn zeo_execute(code: *const c_char) -> *mut c_char {
    let code_str = unsafe {
        if code.is_null() {
            return CString::new("Error: null code").unwrap().into_raw();
        }
        CStr::from_ptr(code)
            .to_str()
            .unwrap_or("")
    };

    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
        match engine.execute_script(code_str) {
            Ok(result) => {
                let result_str = serde_json::to_string(&result).unwrap_or_default();
                CString::new(result_str).unwrap().into_raw()
            }
            Err(e) => {
                error!("Execution error: {}", e);
                let error_str = format!("Error: {}", e);
                CString::new(error_str).unwrap().into_raw()
            }
        }
    } else {
        CString::new("Error: engine not initialized").unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn zeo_call(func: *const c_char, args: *const c_char) -> *mut c_char {
    let func_str = unsafe {
        if func.is_null() {
            return CString::new("Error: null function").unwrap().into_raw();
        }
        CStr::from_ptr(func).to_str().unwrap_or("")
    };

    let args_str = unsafe {
        if args.is_null() {
            return CString::new("Error: null args").unwrap().into_raw();
        }
        CStr::from_ptr(args).to_str().unwrap_or("")
    };

    let args: Vec<serde_json::Value> = match serde_json::from_str(args_str) {
        Ok(a) => a,
        Err(e) => {
            let error_str = format!("Error parsing args: {}", e);
            return CString::new(error_str).unwrap().into_raw();
        }
    };

    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
        match engine.call_function(func_str, args) {
            Ok(result) => {
                let result_str = serde_json::to_string(&result).unwrap_or_default();
                CString::new(result_str).unwrap().into_raw()
            }
            Err(e) => {
                error!("Call error: {}", e);
                let error_str = format!("Error: {}", e);
                CString::new(error_str).unwrap().into_raw()
            }
        }
    } else {
        CString::new("Error: engine not initialized").unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn zeo_set_global(name: *const c_char, value: *const c_char) -> i32 {
    let name_str = unsafe {
        if name.is_null() {
            return -1;
        }
        CStr::from_ptr(name).to_str().unwrap_or("")
    };

    let value_str = unsafe {
        if value.is_null() {
            return -1;
        }
        CStr::from_ptr(value).to_str().unwrap_or("")
    };

    let value: serde_json::Value = match serde_json::from_str(value(value_str) {
        Ok(v) => v,
        Err(e) => {
            error!("Error parsing value: {}", e);
            return -1;
        }
    };

    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
        match engine.set_global(name_str, value) {
            Ok(_) => 0,
            Err(e) => {
                error!("Set global error: {}", e);
                -1
            }
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn zeo_get_global(name: *const c_char) -> *mut c_char {
    let name_str = unsafe {
        if name.is_null() {
            return CString::new("Error: null name").unwrap().into_raw();
        }
        CStr::from_ptr(name).to_str().unwrap_or("")
    };

    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
) {
            let result_str = serde_json::to_string(&result).unwrap_or_default();
            CString::new(result_str).unwrap().into_raw()
        }
        Err(e) => {
            error!("Get global error: {}", e);
            let error_str = format!("Error: {}", e);
            CString::new(error_str).unwrap().into_raw()
        }
    } else {
        CString::new("Error: engine not initialized").unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn zeo_free(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern fn zeo_memory_usage() -> u64 {
    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
        engine.memory_usage()
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn zeo_gc() -> i32 {
    let engine_guard = ENGINE.get()
        .and_then(|e| e.lock().ok())
        .and_then(|e| e.as_ref().cloned());

    if let Some(engine) = engine_guard {
        match engine.gc() {
            Ok(_) => 0,
            Err(e) => {
                error!("GC error: {}", e);
                -1
            }
        }
    } else {
        -1
    }
}

fn main() {
    println!("zeo - High-performance AI Agent execution engine");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
}