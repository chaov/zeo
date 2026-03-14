use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    QuickJS = 0,
    JavaScriptCore = 1,
    V8 = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub memory_limit: usize,
    pub enable_jit: bool,
    pub enable_profiler: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            memory_limit: 16 * 1024 * 1024, // 16MB
            enable_jit: true,
            enable_profiler: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JSValue {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JSValue>),
    Object(std::collections::HashMap<String, JSValue>),
}

impl JSValue {
    pub fn as_string(&self) -> Result<String, EngineError> {
        match self {
            JSValue::String(s) => Ok(s.clone()),
            _ => Err(EngineError::TypeError("Expected string".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub peak: u64,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Engine not initialized")]
    NotInitialized,
}

pub type Result<T> = std::result::Result<T, EngineError>;

pub trait JSEngine: Send + Sync {
    fn new(config: EngineConfig) -> Result<Self> where Self: Sized;
    fn execute(&self, code: &str) -> Result<JSValue>;
    fn eval(&self, code: &str) -> Result<JSValue>;
    fn call(&self, func: &str, args: Vec<JSValue>) -> Result<JSValue>;
    fn set_global(&self, name: &str, value: JSValue) -> Result<()>;
    fn get_global(&self, name: &str) -> Result<JSValue>;
    fn memory_usage(&self) -> MemoryStats;
    fn gc(&self) -> Result<()>;
}

pub struct EngineFactory;

impl EngineFactory {
    pub fn create(engine_type: EngineType, config: EngineConfig) -> Result<Box<dyn JSEngine>> {
        match engine_type {
            #[cfg(feature = "quickjs")]
            EngineType::QuickJS => {
                Ok(Box::new(quickjs_engine::QuickJSEngine::new(config)?))
            }
            #[cfg(not(feature = "quickjs"))]
            EngineType::QuickJS => {
                Err(EngineError::NotImplemented("QuickJS not enabled".to_string()))
            }
            #[cfg(feature = "v8")]
            EngineType::V8 => {
                Ok(Box::new(v8_engine::V8Engine::new(config)?))
            }
            #[cfg(not(feature = "v8"))]
            EngineType::V8 => {
                Err(EngineError::NotImplemented("V8 not enabled".to_string()))
            }
            EngineType::JavaScriptCore => {
                Err(EngineError::NotImplemented("JavaScriptCore not yet implemented".to_string()))
            }
        }
    }
}

#[cfg(feature = "quickjs")]
mod quickjs_engine {
    use super::*;
    use quick_js::{Context, JsValue};

    pub struct QuickJSEngine {
        context: Context,
        config: EngineConfig,
    }

    impl JSEngine for QuickJSEngine {
        fn new(config: EngineConfig) -> Result<Self> {
            let context = Context::new().map_err(|e| 
                EngineError::ExecutionError(format!("Failed to create QuickJS context: {}", e))
            )?;
            
            Ok(Self { context, config })
        }

        fn execute(&self, code: &str) -> Result<JSValue> {
            self.eval(code)
        }

        fn eval(&self, code: &str) -> Result<JSValue> {
            let result = self.context.eval(code).map_err(|e| 
                EngineError::ExecutionError(format!("QuickJS eval error: {}", e))
            )?;
            
            Ok(convert_js_value(result))
        }

        fn call(&self, func: &str, args: Vec<JSValue>) -> Result<JSValue> {
            let args_str = serde_json::to_string(&args)
                .map_err(|e| EngineError::ExecutionError(format!("Failed to serialize args: {}", e)))?;
            
            let call_code = format!("{}(...{})", func, args_str);
            self.eval(&call_code)
        }

        fn set_global(&self, name: &str, value: JSValue) -> Result<()> {
            let value_str = serde_json::to_string(&value)
                .map_err(|e| EngineError::ExecutionError(format!("Failed to serialize value: {}", e)))?;
            
            let code = format!("{} = {};", name, value_str);
            self.context.eval(&code).map_err(|e| 
                EngineError::ExecutionError(format!("Failed to set global: {}", e))
            )?;
            
            Ok(())
        }

        fn get_global(&self, name: &str) -> Result<JSValue> {
            let code = format!("{};", name);
            self.eval(&code)
        }

        fn memory_usage(&self) -> MemoryStats {
            MemoryStats {
                total: self.config.memory_limit as u64,
                used: 0, // QuickJS doesn't expose memory usage
                peak: 0,
            }
        }

        fn gc(&self) -> Result<()> {
            Ok(())
        }
    }

    fn convert_js_value(value: JsValue) -> JSValue {
        match value {
            JsValue::Null => JSValue::Null,
            JsValue::Undefined => JSValue::Undefined,
            JsValue::Bool(b) => JSValue::Bool(b),
            JsValue::Int(i) => JSValue::Number(i as f64),
            JsValue::Float(f) => JSValue::Number(f),
            JsValue::String(s) => JSValue::String(s),
            JsValue::Array(arr) => {
                JSValue::Array(arr.into_iter().map(convert_js_value).collect())
            }
            JsValue::Object(_) => JSValue::Object(std::collections::HashMap::new()),
            JsValue::Function(_) => JSValue::String("[Function]".to_string()),
        }
    }
}

#[cfg(feature = "v8")]
mod v8_engine {
    use super::*;
    
    pub struct V8Engine {
        config: EngineConfig,
    }

    impl JSEngine for V8Engine {
        fn new(config: EngineConfig) -> Result<Self> {
            Ok(Self { config })
        }

        fn execute(&self, code: &str) -> Result<JSValue> {
            Err(EngineError::NotImplemented("V8 execution not yet implemented".to_string()))
        }

        fn eval(&self, code: &str) -> Result<JSValue> {
            Err(EngineError::NotImplemented("V8 eval not yet implemented".to_string()))
        }

        fn call(&self, _func: &str, _args: Vec<JSValue>) -> Result<JSValue> {
            Err(EngineError::NotImplemented("V8 call not yet implemented".to_string()))
        }

        fn set_global(&self, _name: &str, _value: JSValue) -> Result<()> {
            Err(EngineError::NotImplemented("V8 set_global not yet implemented".to_string()))
        }

        fn get_global(&self, _name: &str) -> Result<JSValue> {
            Err(EngineError::NotImplemented("V8 get_global not yet implemented".to_string()))
        }

        fn memory_usage(&self) -> MemoryStats {
            MemoryStats {
                total: self.config.memory_limit as u64,
                used: 0,
                peak: 0,
            }
        }

        fn gc(&self) -> Result<()> {
            Ok(())
        }
    }
}