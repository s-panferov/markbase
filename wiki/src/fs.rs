extern crate notify;

use std::sync::mpsc::{channel};
use std::thread;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{ PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

use actix::prelude::*;

pub enum WatchEvent {
  Update(PathBuf),
  Remove(PathBuf),
  Rename(PathBuf, PathBuf),
}

pub struct WikiWatch {
  pub folder: PathBuf,
  pub rx: Recipient<WatchEvent>
}

impl Actor for WikiWatch {
  type Context = actix::Context<Self>;

  fn started(&mut self, _ctx: &mut Self::Context) {
    let walker = WalkDir::new(&self.folder)
      .follow_links(true)
      .into_iter()
      .filter_map(|e| e.ok());

    for entry in walker {
      self.rx.do_send(WatchEvent::Update(entry.path().to_path_buf())).unwrap()
    }

    let folder = self.folder.to_owned();
    let out = self.rx.clone();
    thread::spawn(move || {
      let (tx, rx) = channel();
      let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100)).unwrap();
      watcher.watch(&folder, RecursiveMode::Recursive).unwrap();
      loop {
        let item = rx.recv();
        if item.is_ok() {
          match item.unwrap() {
            DebouncedEvent::Write(path) => out.do_send(WatchEvent::Update(path)).unwrap(),
            DebouncedEvent::NoticeWrite(_) => (),
            DebouncedEvent::Remove(path) => out.do_send(WatchEvent::Remove(path)).unwrap(),
            DebouncedEvent::NoticeRemove(_) => (),
            DebouncedEvent::Chmod(_) => (),
            DebouncedEvent::Create(path) => out.do_send(WatchEvent::Update(path)).unwrap(),
            DebouncedEvent::Error(_, _) => (),
            DebouncedEvent::Rename(from, to) => out.do_send(WatchEvent::Rename(from, to)).unwrap(),
            DebouncedEvent::Rescan => (),
          };
        } else {
          break;
        }
      }
    });
  }
}

