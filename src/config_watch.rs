// filepath: src/config_watch.rs
use crate::config::NotchConfig;
use calloop::channel::Sender;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn setup_config_watcher(
    tx: Sender<notify::Event>,
) -> Result<notify::RecommendedWatcher, Box<dyn std::error::Error>> {
    let _config_path = NotchConfig::get_config_path();
    let parent = _config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let _config_path = _config_path.canonicalize().unwrap_or(_config_path);

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
