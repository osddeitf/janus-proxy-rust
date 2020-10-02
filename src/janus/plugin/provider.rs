use std::collections::HashMap;
use super::JanusPlugin;
use super::videoroom::VideoRoomPluginFactory;
use crate::janus::core::apierror::*;

pub type BoxedPlugin = Box<dyn JanusPlugin>;

pub trait JanusPluginFactory: Send + Sync {
    fn new(&self) -> BoxedPlugin;
}

/** Provide singleton instance of each registered plugins */
pub struct JanusPluginProvider {
    /// Store mapping from 'name' to 'factory' function - create new instance out of thin air
    plugins: HashMap<String, Box<dyn JanusPluginFactory>>
}

impl JanusPluginProvider {
    /** For customization */
    pub fn empty() -> JanusPluginProvider {
        JanusPluginProvider {
            plugins: HashMap::new()
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
    pub fn resolve(&self, name: String) -> Result<BoxedPlugin, JanusError> {
        let factory = match self.plugins.get(&name) {
            Some(x) => x,
            None => return Err(JanusError::new(JANUS_ERROR_PLUGIN_NOT_FOUND, format!("No such plugin '{}'", name)))
        };

        let instance = factory.new();
        Ok(instance)
    }
}
