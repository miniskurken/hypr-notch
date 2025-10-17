// filepath: src/config_watch.rs
use crate::config::NotchConfig;
use calloop::channel::{channel, Sender};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn setup_config_watcher(
    tx: Sender<notify::Event>,
) -> Result<notify::RecommendedWatcher, Box<dyn std::error::Error>> {
    let config_path = NotchConfig::get_config_path();
    let parent = config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let config_path = config_path.canonicalize().unwrap_or(config_path);

    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )?;
    watcher.watch(&parent, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
