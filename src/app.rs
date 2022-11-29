use leveldb::{
    self,
    kv::KV,
    options::{ReadOptions, WriteOptions},
};
use minikeyvaluer::Record;
use std::collections::HashMap;

pub struct App {
    pub db: leveldb::database::Database<i32>,
    pub mlock: std::sync::Mutex<bool>,
    pub lock: HashMap<String, ()>,

    // Params
    pub uploadids: HashMap<String, bool>,
    pub volumes: Vec<String>,
    pub fallback: String,
    pub replicas: usize,
    pub subvolumes: usize,
    pub protect: bool,
    pub md5sum: bool,
    pub voltimeout: std::time::Duration,
}
impl App {
    pub fn bytes_to_i32(bytes: &[u8]) -> i32 {
        let mut ret = 0;
        for b in bytes {
            ret = ret * 256 + *b as i32;
        }
        return ret;
    }
    pub fn getRecord(&self, key: &[u8]) -> Option<Record> {
        let key_i32 = App::bytes_to_i32(&key);
        let lock = self.mlock.lock().unwrap();
        let result = self.db.get(ReadOptions::new(), key_i32).unwrap()?;
        let rec: Record = result.as_slice().into();
        Some(rec)
    }
    pub fn putRecord(&self, key: &[u8], rec: Record) -> bool {
        let key_i32 = App::bytes_to_i32(&key);
        let value: Vec<u8> = rec.into();
        self.db.put(WriteOptions::new(), key_i32, &value).unwrap();
        true
    }
    pub fn unlockKey(&mut self, key: &[u8]) {
        let key_i32 = App::bytes_to_i32(&key);
        let _lock = self.mlock.lock().unwrap();
        self.lock.remove(&key_i32.to_string());
    }
    pub fn lockKey(&mut self, key: &[u8]) -> bool {
        let key_i32 = App::bytes_to_i32(&key);
        let _lock = self.mlock.lock().unwrap();
        if self.lock.contains_key(&key_i32.to_string()) {
            return false;
        }
        self.lock.insert(key_i32.to_string(), ());
        true
    }
}
