use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{anyhow, bail, Context, Result};
use crate::paths::ResolvedPaths;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Defaults {
    #[serde(default)]
    pub binary: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Profile {
    #[serde(default)]
    pub binary: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveProfile {
    pub name: String,
    pub binary: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum LoadedConfig {
    Missing { expected_path: PathBuf },
    Loaded(Config),
}

impl Config {
    pub fn load(resolved_paths: &ResolvedPaths) -> Result<LoadedConfig> {
        if !resolved_paths.config_file.exists() {
            return Ok(LoadedConfig::Missing {
                expected_path: resolved_paths.config_file.clone(),
            });
        }

        let config = Self::from_path(&resolved_paths.config_file)
            .with_context(|| format!("failed to parse {}", resolved_paths.config_file.display()))?;
        Ok(LoadedConfig::Loaded(config))
    }

    pub fn from_yaml_str(text: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(text)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Self::from_yaml_str(&text)
    }

    pub fn validate(&self) -> Result<()> {
        if self.profiles.is_empty() {
            bail!("config must define at least one profile in 'profiles'");
        }

        for name in self.profiles.keys() {
            validate_profile_name(name)?;
        }

        Ok(())
    }

    pub fn effective_profile(&self, name: &str) -> Result<EffectiveProfile> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| anyhow!(unknown_profile_message(name, self.profile_names())))?;

        let mut env = self.defaults.env.clone();
        env.extend(profile.env.clone());

        let mut args = self.defaults.args.clone();
        args.extend(profile.args.clone());

        Ok(EffectiveProfile {
            name: name.to_owned(),
            binary: profile
                .binary
                .clone()
                .or_else(|| self.defaults.binary.clone())
                .unwrap_or_else(|| "goose".to_owned()),
            args,
            env,
        })
    }

    pub fn profile_names(&self) -> Vec<&str> {
        self.profiles.keys().map(String::as_str).collect()
    }
}

pub fn validate_profile_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("invalid profile name '': expected ^[a-z0-9][a-z0-9_-]*$");
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        bail!("invalid profile name '{name}': expected ^[a-z0-9][a-z0-9_-]*$");
    }

    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-') {
        bail!("invalid profile name '{name}': expected ^[a-z0-9][a-z0-9_-]*$");
    }

    Ok(())
}

pub fn missing_config_message(expected_path: &Path) -> String {
    format!(
        "No geese config found at {}.\n\nCreate it with:\n\n{}",
        expected_path.display(),
        include_str!("../config.example.yml")
    )
}

pub fn unknown_profile_message(name: &str, available: Vec<&str>) -> String {
    if available.is_empty() {
        format!("unknown profile '{name}'")
    } else {
        format!(
            "unknown profile '{name}'. Available profiles: {}",
            available.join(", ")
        )
    }
}
