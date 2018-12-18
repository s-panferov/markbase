extern crate notify;

use std::sync::mpsc::{channel, Receiver};
use std::thread;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

pub enum WatchEvent {
  Update(PathBuf),
  Remove(PathBuf),
  Rename(PathBuf, PathBuf),
}

pub fn scan(folder: &Path) -> Receiver<PathBuf> {
  // Create a channel to receive the events.
  let (tx, rx) = channel::<PathBuf>();
  let folder = folder.to_owned();
  thread::spawn(move || {
    let walker = WalkDir::new(folder)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok());

    for entry in walker {
      tx.send(entry.path().to_path_buf()).unwrap();
    }
  });

  rx
}

pub fn watch(folder: &Path) -> Receiver<WatchEvent> {
  let (tx_out, rx_out) = channel::<WatchEvent>();

  let folder = folder.to_owned();
  thread::spawn(move || {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(&folder, RecursiveMode::Recursive).unwrap();
    loop {
      let item = rx.recv();
      if item.is_ok() {
        match item.unwrap() {
          DebouncedEvent::Write(path) => tx_out.send(WatchEvent::Update(path)).unwrap(),
          DebouncedEvent::NoticeWrite(_) => (),
          DebouncedEvent::Remove(path) => tx_out.send(WatchEvent::Remove(path)).unwrap(),
          DebouncedEvent::NoticeRemove(_) => (),
          DebouncedEvent::Chmod(_) => (),
          DebouncedEvent::Create(path) => tx_out.send(WatchEvent::Update(path)).unwrap(),
          DebouncedEvent::Error(_, _) => (),
          DebouncedEvent::Rename(from, to) => tx_out.send(WatchEvent::Rename(from, to)).unwrap(),
          DebouncedEvent::Rescan => (),
        };
      } else {
        break;
      }
    }
  });

  rx_out
}
