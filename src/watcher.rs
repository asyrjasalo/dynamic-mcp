use anyhow::Result;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::Path;
use tokio::sync::mpsc;

pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn new(config_path: &Path) -> Result<(Self, mpsc::Receiver<()>)> {
        let (tx, rx) = mpsc::channel(100);

        let config_path_buf = config_path.to_path_buf();
        let watch_path = config_path_buf.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| match res {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    if event.paths.iter().any(|p| p == &watch_path) {
                        tracing::info!("Config file changed: {:?}, triggering reload", event.kind);
                        let _ = tx.blocking_send(());
                    }
                }
                _ => {}
            },
            Err(e) => {
                tracing::warn!("File watch error: {}", e);
            }
        })?;

        watcher.configure(Config::default())?;
        watcher.watch(&config_path_buf, RecursiveMode::NonRecursive)?;

        tracing::info!("Watching config file: {:?}", config_path_buf);

        Ok((Self { _watcher: watcher }, rx))
    }
}
