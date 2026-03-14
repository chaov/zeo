use crate::engine::{JSEngine, JSValue, Result, EngineError};
use std::sync::Arc;

pub fn setup(engine: Arc<dyn JSEngine>) -> Result<()> {
    engine.set_global("openclaw", create_openclaw_object(engine.clone()))?;
    Ok(())
}

fn create_openclaw_object(engine: Arc<dyn JSEngine>) -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("version".to_string(), JSValue::String("1.0.0".to_string()));
        map.insert("gateway".to_string(), create_gateway_object(engine));
        map.insert("agent".to_string(), create_agent_object(engine));
        map.insert("tools".to_string(), create_tools_object(engine));
    }
    
    obj
}

fn create_gateway_object(engine: Arc<dyn JSEngine>) -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("connect".to_string(), JSValue::String("[Function: connect]".to_string()));
        map.insert("send".to_string(), JSValue::String("[Function: send]".to_string()));
        map.insert("disconnect".to_string(), JSValue::String("[Function: disconnect]".to_string()));
    }
    
    obj
}

fn create_agent_object(engine: Arc<dyn JSEngine>) -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("execute".to_string(), JSValue::String("[Function: execute]".to_string()));
        map.insert("onMessage".to_string(), JSValue::String("[Function: onMessage]".to_string()));
        map.insert("onToolCall".to_string(), JSValue::String("[Function: onToolCall]".to_string()));
    }
    
    obj
}

fn create_tools_object(engine: Arc<dyn JSEngine>) -> JSValue {
    let mut obj = JSValue::Object(std::collections::HashMap::new());
    
    if let JSValue::Object(ref mut map) = obj {
        map.insert("register".to_string(), JSValue::String("[Function: register]".to_string()));
        map.insert("execute".to_string(), JSValue::String("[Function: execute]".to_string()));
        map.insert("list".to_string(), JSValue::String("[Function: list]".to_string()));
    }
    
    obj
}