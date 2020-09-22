use crate::janus::error::JanusError;
use crate::janus::videoroom::VideoRoomPluginFactory;
use crate::janus::error::code::*;
use crate::janus::json::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::janus::core::JanusHandle;

// Resemble `janus_videoroom_handle_message` function signature
pub struct JanusPluginMessage {
    pub handle: Arc<JanusHandle>,
    pub transaction: String,
    pub body: String,
    pub jsep: Option<JSON_OBJECT>
}

impl JanusPluginMessage {
    pub fn new(handle: Arc<JanusHandle>, transaction: String, body: String, jsep: Option<JSON_OBJECT>) -> JanusPluginMessage {
        JanusPluginMessage { handle, transaction, body, jsep }
    }
}

// * functions in traits cannot be declared `async`
// May `handle_message*` return JanusError? TODO
pub trait JanusPlugin: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn handle_message(&self, message: JanusPluginMessage) -> JanusPluginResult;
    fn handle_async_message(&self, message: JanusPluginMessage) -> JanusPluginResult;
    // fn set_opaque_id(&mut self, opaque_id: &str);
}

#[allow(non_camel_case_types, dead_code)]
pub enum JanusPluginResultType {
    // 'Shutting down' or 'Plugin not initialized'
    JANUS_PLUGIN_ERROR,
    JANUS_PLUGIN_OK,
    JANUS_PLUGIN_OK_WAIT
}

pub struct JanusPluginResult {
    pub kind: JanusPluginResultType,     // 'type' is reserved
    pub text: Option<String>,
    pub content: Option<JSON_OBJECT>
}

#[allow(dead_code)]
impl JanusPluginResult {
    pub fn ok(data: JSON_OBJECT) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK,
            text: None, content: Some(data)
        }
    }

    pub fn wait(text: Option<String>) -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_OK_WAIT,
            text, content: None
        }
    }

    pub fn err() -> JanusPluginResult {
        JanusPluginResult {
            kind: JanusPluginResultType::JANUS_PLUGIN_ERROR,
            text: None, content: None
        }
    }
}

pub type BoxedPlugin = Box<dyn JanusPlugin>;

pub trait JanusPluginFactory: Send + Sync {
    fn new(&self) -> BoxedPlugin;
}

/** Provide singleton instance of each registered plugins */
pub struct JanusPluginProvider {
    /// Store mapping from 'name' to 'factory' function - create new instance out of thin air
    plugins: HashMap<String, Box<dyn JanusPluginFactory>>,
    instance: Mutex<HashMap<String, Arc<BoxedPlugin>>>
}

impl JanusPluginProvider {
    /** For customization */
    pub fn empty() -> JanusPluginProvider {
        JanusPluginProvider {
            plugins: HashMap::new(),
            instance: Mutex::new(HashMap::new())
        }
    }

    /** Default configured */
    pub fn default() -> JanusPluginProvider {
        let provider = Self::empty();
        provider.add(String::from("janus.plugin.videoroom"), Box::new(VideoRoomPluginFactory))
    }

    pub fn add(mut self, name: String, factory: Box<dyn JanusPluginFactory>) -> JanusPluginProvider {
        self.plugins.insert(name, factory);
        self
    }

    /** Resolve plugin by name */
    pub fn resolve(&self, name: String) -> Result<Arc<BoxedPlugin>, JanusError> {
        let factory = match self.plugins.get(&name) {
            Some(x) => x,
            None => return Err(JanusError::new(JANUS_ERROR_PLUGIN_NOT_FOUND, format!("No such plugin '{}'", name)))
        };

        let mut map = self.instance.lock().unwrap();
        let instance = match map.get(&name) {
            Some(instance) => instance.clone(),
            None => {
                println!("LOG: Construct new instance of '{}' plugin", name);
                let instance = Arc::new(factory.new());
                map.insert(name, instance.clone());
                instance
            }
        };

        Ok(instance)
    }
}
