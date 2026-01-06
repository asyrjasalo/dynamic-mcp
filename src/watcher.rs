use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn new(config_path: &Path) -> Result<(Self, mpsc::Receiver<()>)> {
        let (tx, rx) = mpsc::channel(100);

        let config_path = config_path.to_path_buf();
        let tx = Arc::new(tx);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                match event.kind {
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                        tracing::info!("Config file changed, triggering reload");
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let _ = tx.send(()).await;
                        });
                    }
                    _ => {}
                }
            }
        })?;

        watcher.configure(Config::default())?;
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

        tracing::info!("Watching config file: {:?}", config_path);

        Ok((Self { _watcher: watcher }, rx))
    }
}
