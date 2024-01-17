use std::collections::HashMap;
use std::sync::Arc;
use lazy_static::lazy_static;
use tokio::process::Child;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref LOGS: Arc<Mutex<HashMap<i64, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref CHILDS: Arc<Mutex<HashMap<u32, ChildProcess>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub struct ChildProcess {
    pub child: Child,
    pub group_id: u32
}


