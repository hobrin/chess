use std::time::{SystemTime, UNIX_EPOCH};
use hashbrown::HashMap;

extern crate lazy_static;
use lazy_static::lazy_static;
use std::sync::Mutex;

const ON: bool = false;

pub struct Profiler {
    name: String,
    time_started: SystemTime,
}
impl Profiler {
    pub fn stop(&self) {
        if ON {
            let elapsed = self.time_started.elapsed().unwrap().as_nanos();
            add_time(&self.name, elapsed);
        }
    }
}

type Dictionary = HashMap<String, u128>;

lazy_static! {
    static ref TIME_TABLE: Mutex<Dictionary> = Mutex::new(HashMap::new());
}

fn add_time(name: &String, elapsed: u128) {
    if ON {
        let mut table = TIME_TABLE.lock().unwrap();
        *table.get_mut(name).unwrap() += elapsed;
    }
}

pub fn start_timing(name: &str) -> Profiler {
    if ON {
        let mut table = TIME_TABLE.lock().unwrap();
        if !table.contains_key(name) {
            table.insert(name.to_string(), 0);
        }
        Profiler {name: name.to_string(), time_started: SystemTime::now()}
    } else {
        Profiler {name: name.to_string(), time_started: SystemTime::UNIX_EPOCH}
    }
}

pub fn reset() {
    TIME_TABLE.lock().unwrap().clear();
}
pub fn print() {
    let mut table = TIME_TABLE.lock().unwrap();
    for (name, elapsed) in table.drain() {
        println!("{}: Took {}ms", name, elapsed/1000_000);
    }
}