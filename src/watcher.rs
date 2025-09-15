use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

use crate::{FaceRecognitionError, Result};

pub struct FolderWatcher {
    watcher: Option<RecommendedWatcher>,
    receiver: Option<mpsc::Receiver<notify::Result<Event>>>,
}

impl FolderWatcher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            watcher: None,
            receiver: None,
        })
    }

    pub fn start_watching<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| FaceRecognitionError::WatchError(e.to_string()))?;

        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| FaceRecognitionError::WatchError(e.to_string()))?;

        self.watcher = Some(watcher);
        self.receiver = Some(rx);

        info!("Started watching directory: {}", path.as_ref().display());
        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.receiver = None;
        info!("Stopped watching directory");
    }

    pub async fn watch_for_changes<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        let receiver = self
            .receiver
            .take()
            .ok_or_else(|| FaceRecognitionError::WatchError("Watcher not started".to_string()))?;

        tokio::task::spawn_blocking(move || {
            let mut last_change_time = SystemTime::now();

            loop {
                match receiver.recv_timeout(Duration::from_secs(1)) {
                    Ok(Ok(event)) => {
                        debug!("File system event: {:?}", event);

                        // Filter for relevant events (file modifications/creations)
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                let now = SystemTime::now();
                                // Debounce events - only trigger if more than 2 seconds have passed
                                if now
                                    .duration_since(last_change_time)
                                    .unwrap_or(Duration::from_secs(0))
                                    > Duration::from_secs(2)
                                {
                                    info!("Database folder changed, triggering reload...");
                                    callback();
                                    last_change_time = now;
                                }
                            }
                            _ => {
                                debug!("Ignoring event: {:?}", event.kind);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("File watcher error: {}", e);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue watching
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("File watcher disconnected");
                        break;
                    }
                }
            }
        })
        .await
        .map_err(|e| FaceRecognitionError::WatchError(e.to_string()))?;

        Ok(())
    }
}

impl Drop for FolderWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

pub fn get_latest_mod_time<P: AsRef<Path>>(path: P) -> Result<SystemTime> {
    let mut latest_time = SystemTime::UNIX_EPOCH;

    fn visit_dir(dir: &Path, latest: &mut SystemTime) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, latest)?;
                } else {
                    let metadata = entry.metadata()?;
                    let modified = metadata.modified()?;
                    if modified > *latest {
                        *latest = modified;
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(path.as_ref(), &mut latest_time)?;
    Ok(latest_time)
}
