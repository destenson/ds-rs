use crate::config::{AppConfig, VideoSourceConfig};
use crate::error::{Result, SourceVideoError};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait ConfigLoader: Send + Sync {
    fn load(&self, path: &Path) -> Result<AppConfig>;
    fn validate(&self, config: &AppConfig) -> Result<()>;
    fn load_partial(&self, path: &Path, base: &AppConfig) -> Result<AppConfig>;
}

pub struct TomlConfigLoader {
    validator: Arc<dyn ConfigValidator>,
}

impl TomlConfigLoader {
    pub fn new(validator: Arc<dyn ConfigValidator>) -> Self {
        Self { validator }
    }
}

impl ConfigLoader for TomlConfigLoader {
    fn load(&self, path: &Path) -> Result<AppConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| SourceVideoError::config(format!("Failed to parse TOML: {}", e)))?;

        self.validate(&config)?;
        Ok(config)
    }

    fn validate(&self, config: &AppConfig) -> Result<()> {
        self.validator.validate(config)
    }

    fn load_partial(&self, path: &Path, base: &AppConfig) -> Result<AppConfig> {
        if !path.exists() {
            return Ok(base.clone());
        }

        let content = std::fs::read_to_string(path)?;

        // Parse as toml::Value to allow partial updates
        let partial: toml::Value = toml::from_str(&content).map_err(|e| {
            SourceVideoError::config(format!("Failed to parse partial TOML: {}", e))
        })?;

        // Serialize base config to toml::Value
        let mut base_value = toml::Value::try_from(base.clone()).map_err(|e| {
            SourceVideoError::config(format!("Failed to serialize base config: {}", e))
        })?;

        // Merge partial into base
        merge_toml_values(&mut base_value, partial);

        // Deserialize back to AppConfig
        let merged: AppConfig = base_value.try_into().map_err(|e| {
            SourceVideoError::config(format!("Failed to deserialize merged config: {}", e))
        })?;

        self.validate(&merged)?;
        Ok(merged)
    }
}

fn merge_toml_values(base: &mut toml::Value, partial: toml::Value) {
    match (base, partial) {
        (toml::Value::Table(base_table), toml::Value::Table(partial_table)) => {
            for (key, value) in partial_table {
                match base_table.get_mut(&key) {
                    Some(base_value) => merge_toml_values(base_value, value),
                    None => {
                        base_table.insert(key, value);
                    }
                }
            }
        }
        (base, partial) => *base = partial,
    }
}

pub trait ConfigValidator: Send + Sync {
    fn validate(&self, config: &AppConfig) -> Result<()>;
    fn validate_source(&self, source: &VideoSourceConfig) -> Result<()>;
}

pub struct AtomicConfigLoader {
    loader: Arc<dyn ConfigLoader>,
    current: Arc<RwLock<AppConfig>>,
}

impl AtomicConfigLoader {
    pub fn new(loader: Arc<dyn ConfigLoader>, initial: AppConfig) -> Self {
        Self {
            loader,
            current: Arc::new(RwLock::new(initial)),
        }
    }

    pub async fn load_atomic(&self, path: &Path) -> Result<AppConfig> {
        // Load new config
        let new_config = self.loader.load(path)?;

        // Atomically update current config
        let mut current = self.current.write().await;
        *current = new_config.clone();

        Ok(new_config)
    }

    pub async fn get_current(&self) -> AppConfig {
        self.current.read().await.clone()
    }

    pub async fn update_if_valid<F>(&self, update_fn: F) -> Result<AppConfig>
    where
        F: FnOnce(&AppConfig) -> Result<AppConfig>,
    {
        let mut current = self.current.write().await;
        let new_config = update_fn(&*current)?;

        // Validate before updating
        self.loader.validate(&new_config)?;

        *current = new_config.clone();
        Ok(new_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::validator::DefaultConfigValidator;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_toml_loader() {
        let validator = Arc::new(DefaultConfigValidator::new());
        let loader = TomlConfigLoader::new(validator);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
            log_level = "debug"
            
            [server]
            port = 8555
            address = "127.0.0.1"
        "#
        )
        .unwrap();

        let config = loader.load(temp_file.path()).unwrap();
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.server.port, 8555);
    }

    #[test]
    fn test_partial_config_merge() {
        let mut base = toml::Value::Table(toml::map::Map::new());
        base.as_table_mut().unwrap().insert(
            "server".to_string(),
            toml::Value::Table({
                let mut table = toml::map::Map::new();
                table.insert("port".to_string(), toml::Value::Integer(8554));
                table.insert(
                    "address".to_string(),
                    toml::Value::String("0.0.0.0".to_string()),
                );
                table
            }),
        );

        let partial = toml::Value::Table({
            let mut table = toml::map::Map::new();
            table.insert(
                "server".to_string(),
                toml::Value::Table({
                    let mut inner = toml::map::Map::new();
                    inner.insert("port".to_string(), toml::Value::Integer(9000));
                    inner
                }),
            );
            table
        });

        merge_toml_values(&mut base, partial);

        let server = base
            .as_table()
            .unwrap()
            .get("server")
            .unwrap()
            .as_table()
            .unwrap();
        assert_eq!(server.get("port").unwrap().as_integer().unwrap(), 9000);
        assert_eq!(server.get("address").unwrap().as_str().unwrap(), "0.0.0.0");
    }
}
