//! Config Watcher - Watch config file for changes and reload
//!
//! This module provides hot-reload of config file without restarting the agent.

use std::path::PathBuf;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

#[derive(Debug, Clone)]
pub enum ConfigChange {
    Modified(PathBuf),
    Created(PathBuf),
    Removed(PathBuf),
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf, on_change: impl Fn(ConfigChange) + Send + 'static) -> Result<Self, notify::Error> {
        let (tx, mut rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                for path in event.paths {
                    let change = match event.kind {
                        notify::EventKind::Create(_) => ConfigChange::Created(path.clone()),
                        notify::EventKind::Modify(_) => ConfigChange::Modified(path.clone()),
                        notify::EventKind::Remove(_) => ConfigChange::Removed(path.clone()),
                        _ => continue,
                    };
                    
                    info!(path = %path.display(), kind = ?change, "Config file changed");
                    on_change(change);
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}

pub fn watch_config_file(
    config_path: Option<PathBuf>,
    on_change: impl Fn(ConfigChange) + Send + 'static + Clone,
) -> Option<ConfigWatcher> {
    let config_path = config_path.unwrap_or_else(|| {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("escluse-agent").join("agent.json")
    });
    
    if !config_path.exists() {
        warn!(path = %config_path.display(), "Config file not found, skipping watcher");
        return None;
    }

    match ConfigWatcher::new(config_path.clone(), on_change) {
        Ok(watcher) => {
            info!("Config watcher started");
            Some(watcher)
        }
        Err(e) => {
            error!(error = %e, "Failed to start config watcher");
            None
        }
    }
}
