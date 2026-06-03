use super::config::RuntimeConfig;

pub trait EnvReader: Send + Sync {
    fn var(&self, key: &str) -> Option<String>;
}

pub trait AppContext: Send + Sync {
    fn config(&self) -> &RuntimeConfig;
    fn env(&self) -> &dyn EnvReader;
}

pub struct ProcessEnv;

impl EnvReader for ProcessEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

pub struct AppState {
    config: RuntimeConfig,
    env: ProcessEnv,
}

impl AppState {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            env: ProcessEnv,
        }
    }
}

impl AppContext for AppState {
    fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    fn env(&self) -> &dyn EnvReader {
        &self.env
    }
}
