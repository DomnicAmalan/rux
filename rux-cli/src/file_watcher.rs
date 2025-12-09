use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashSet;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watched_files: HashSet<PathBuf>,
}

impl FileWatcher {
    pub fn new() -> notify::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher = notify::recommended_watcher(tx)?;
        
        Ok(Self {
            watcher,
            receiver: rx,
            watched_files: HashSet::new(),
        })
    }
    
    pub fn watch_directory(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }
    
    pub fn watch_file(&mut self, path: &Path) -> notify::Result<()> {
        if let Some(parent) = path.parent() {
            self.watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }
        self.watched_files.insert(path.to_path_buf());
        Ok(())
    }
    
    pub fn check_for_changes(&self) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();
        
        // Non-blocking check for file changes
        while let Ok(Ok(event)) = self.receiver.try_recv() {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("rsx") {
                            changed_files.push(path);
                        }
                    }
                }
                _ => {}
            }
        }
        
        changed_files
    }
    
    pub fn wait_for_change(&self) -> notify::Result<Vec<PathBuf>> {
        match self.receiver.recv() {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        let mut changed = Vec::new();
                        for path in event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("rsx") {
                                changed.push(path);
                            }
                        }
                        Ok(changed)
                    }
                    _ => Ok(Vec::new()),
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(Vec::new()),
        }
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create file watcher")
    }
}
