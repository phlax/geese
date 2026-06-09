use std::env;
use std::path::PathBuf;

use crate::error::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub config_file: PathBuf,
    pub data_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProfilePaths {
    pub root: PathBuf,
    pub xdg_config_home: PathBuf,
    pub xdg_data_home: PathBuf,
    pub xdg_state_home: PathBuf,
    pub xdg_cache_home: PathBuf,
    pub bin_dir: PathBuf,
    pub goose_config_dir: PathBuf,
    pub symlink_path: PathBuf,
}

impl ProfilePaths {
    pub fn ensure_dirs(&self) -> Result<()> {
        for path in [
            &self.root,
            &self.xdg_config_home,
            &self.xdg_data_home,
            &self.xdg_state_home,
            &self.xdg_cache_home,
            &self.bin_dir,
            &self.goose_config_dir,
        ] {
            std::fs::create_dir_all(path)
                .with_context(|| format!("failed to create {}", path.display()))?;
        }
        Ok(())
    }

    pub fn app_id(&self, profile_name: &str) -> String {
        format!("goose-{profile_name}")
    }
}

pub fn resolve_paths() -> Result<ResolvedPaths> {
    let config_file = match env::var_os("GEESE_CONFIG") {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(anyhow!(
                    "$GEESE_CONFIG must be an absolute path, got {}",
                    path.display()
                ));
            }
            path
        }
        None => dirs::config_dir()
            .ok_or_else(|| anyhow!("could not resolve config directory"))?
            .join("geese")
            .join("config.yml"),
    };

    let data_root = dirs::data_dir()
        .ok_or_else(|| anyhow!("could not resolve data directory"))?
        .join("geese");

    Ok(ResolvedPaths {
        config_file,
        data_root,
    })
}

pub fn profile_paths(paths: &ResolvedPaths, profile_name: &str) -> ProfilePaths {
    let root = paths.data_root.join(profile_name);
    let xdg_config_home = root.join("config");
    let xdg_data_home = root.join("data");
    let xdg_state_home = root.join("state");
    let xdg_cache_home = root.join("cache");
    let bin_dir = root.join("bin");
    let goose_config_dir = xdg_config_home.join("goose");
    let symlink_path = bin_dir.join(format!("goose-{profile_name}"));

    ProfilePaths {
        root,
        xdg_config_home,
        xdg_data_home,
        xdg_state_home,
        xdg_cache_home,
        bin_dir,
        goose_config_dir,
        symlink_path,
    }
}
