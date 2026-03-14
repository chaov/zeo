use crate::engine::{JSEngine, JSValue, Result, EngineError};
use std::sync::Arc;

pub fn setup(engine: Arc<dyn JSEngine>) -> Result<()> {
    engine.set_global("process", create_process_object())?;
    engine.set_global("console", create_console_object())?;
    engine.set_global("Buffer", create_buffer_object())?;
    engine.set_global("require", JSValue::String("[Function: require]".to_string()))?;
    engine.set_global("__filename", JSValue::String("".to_string()))?;
    engine.set_global("__dirname", JSValue::String("".to_string()))?;
    engine.set_global("module", create_module_object())?;
    engine.set_global("exports", JSValue::Object(std::collections::HashMap::new()))?;
    
    Ok(())
}

fn create_process_object() -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("version".to_string(), JSValue::String(env!("CARGO_PKG_VERSION").to_string()));
        map.insert("platform".to_string(), JSValue::String(std::env::consts::OS.to_string()));
        map.insert("arch".to_string(), JSValue::String(std::env::consts::ARCH.to_string()));
        map.insert("pid".to_string(), JSValue::Number(std::process::id() as f64));
        map.insert("env".to_string(), create_env_object());
        map.insert("argv".to_string(), create_argv_array());
    }
    
    obj
}

fn create_env_object() -> JSValue {
    let mut env_map = std::collections::HashMap::new();
    
    for (key, value) in std::env::vars() {
        env_map.insert(key, JSValue::String(value));
    }
    
    JSValue::Object(env_map)
}

fn create_argv_array() -> JSValue {
    let args: Vec<JSValue> = std::env::args()
        .map(|arg| JSValue::String(arg))
        .collect();
    
    JSValue::Array(args)
}

fn create_console_object() -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("log".to_string(), JSValue::String("[Function: log]".to_string()));
        map.insert("error".to_string(), JSValue::String("[Function: error]".to_string()));
        map.insert("warn".to_string(), JSValue::String("[Function: warn]".to_string()));
        map.insert("info".to_string(), JSValue::String("[Function: info]".to_string()));
        map.insert("debug".to_string(), JSValue::String("[Function: debug]".to_string()));
    }
    
    obj
}

fn create_buffer_object() -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("from".to_string(), JSValue::String("[Function: from]".to_string()));
        map.insert("alloc".to_string(), JSValue::String("[Function: alloc]".to_string()));
        map.insert("byteLength".to_string(), JSValue::String("[Function: byteLength]".to_string()));
    }
    
    obj
}

fn create_module_object() -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("exports".to_string(), JSValue::Object(std::collections::HashMap::new()));
        map.insert("require".to_string(), JSValue::String("[Function: require]".to_string()));
        map.insert("id".to_string(), JSValue::String("".to_string()));
        map.insert("filename".to_string(), JSValue::String("".to_string()));
        map.insert("loaded".to_string(), JSValue::Bool(false));
    }
    
    obj
}