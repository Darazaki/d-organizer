use crate::config::{UserConfig, Rule};
use std::sync::mpsc::{channel, Receiver};
use notify::{raw_watcher, RecursiveMode, Watcher, RecommendedWatcher, RawEvent};
use std::thread;
use std::time::Duration;
use crate::file::File;

pub struct Notifier{
    watcher: RecommendedWatcher,
    receiver: Receiver<RawEvent>,
}

impl Notifier {
    pub fn new() -> Notifier {
        let (sender, receiver) = channel();
        let watcher = raw_watcher(sender).unwrap();

        Notifier { watcher, receiver }
    }

    pub fn watch(&mut self, user_config: UserConfig) {
        self.watcher
            .watch(user_config.args.watch, RecursiveMode::Recursive)
            .unwrap();

        loop {
            match self.receiver.recv() {
                Ok(RawEvent {
                       path: Some(abs_path),
                       op: Ok(op),
                       cookie: _,
                   }) => match op {
                    notify::op::CREATE => {
                        if abs_path.is_file() {
                            let extension = abs_path.extension().unwrap().to_str().unwrap();
                            let rule = Rule::from_yaml(&user_config.rules[extension]);
                            let file = File::from(&abs_path);
                            if !(rule.is_null() || rule.is_badvalue()) {
                                let dst = rule.get_dst_for_file(&file);
                                thread::sleep(Duration::from_millis(user_config.args.delay));
                                file.rename(dst).unwrap();
                            }
                        }
                    },
                    _ => continue,
                },
                Ok(event) => eprintln!("broken event: {:?}", event),
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }

    }
}