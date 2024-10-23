use crate::state::THREADS;
use crate::types::ThreadStatus;
use dod_cpu::threads::get_available_threads;
use std::collections::BTreeMap;

pub struct ThreadsManager {
    pub max_threads: u32,
    pub t_map: BTreeMap<u32, ThreadStatus>,
}

impl Default for ThreadsManager {
    fn default() -> Self {
        let max_threads = get_available_threads();
        let mut t_map = BTreeMap::new();
        for thread in 0..max_threads {
            t_map.entry(thread).or_insert(ThreadStatus::Idle);
        }
        ThreadsManager {
            max_threads: get_available_threads(),
            t_map,
        }
    }
}

impl ThreadsManager {
    #[allow(dead_code)]
    pub fn get_available_threads() -> u32 {
        get_available_threads()
    }

    pub fn set_max_threads(&mut self, max_threads: u32) {
        self.max_threads = max_threads;
        let mut t_map = BTreeMap::new();
        for thread in 0..max_threads {
            t_map.entry(thread).or_insert(ThreadStatus::Idle);
        }
        self.t_map = t_map;
    }
    pub fn get_t(&self, t: u32) -> Option<ThreadStatus> {
        self.t_map.get(&t).map(|e| e.clone())
    }
    pub fn set_t(&mut self, t: u32, status: ThreadStatus) -> Option<ThreadStatus> {
        self.t_map.insert(t, status)
    }

    pub fn get_idle_ts(&self) -> Vec<u32> {
        let mut idle_ts = vec![];
        for (t, status) in self.t_map.iter() {
            if status == &ThreadStatus::Idle {
                idle_ts.push(*t);
            }
        }
        idle_ts
    }
    pub fn get_all_ts(&self) -> BTreeMap<u32, ThreadStatus> {
        self.t_map.clone()
    }
}
pub async fn tm_get_all_ts() -> Vec<u32> {
    let tm = THREADS.lock().await;
    let mut res = tm.get_all_ts().keys().cloned().collect::<Vec<u32>>();
    res.sort();
    res
}

pub async fn tm_get_available_threads() -> u32 {
    let tm = THREADS.lock().await;
    tm.max_threads
}
pub async fn tm_get_idle_ts() -> Vec<u32> {
    let tm = THREADS.lock().await;
    let mut res = tm.get_idle_ts();
    res.sort();
    res
}

pub async fn tm_set_t(t: u32, status: ThreadStatus) -> Option<ThreadStatus> {
    let mut tm = THREADS.lock().await;
    tm.set_t(t, status)
}

#[allow(dead_code)]
pub async fn tm_get_t(t: u32) -> Option<ThreadStatus> {
    let tm = THREADS.lock().await;
    tm.get_t(t)
}

#[test]
fn test_get_threads() {
    let t = ThreadsManager::get_available_threads();
    assert_eq!(t > 0, true);
}

#[test]
fn test_set_threads() {
    let mut tm = ThreadsManager::default();
    let t1_token = "some_token".to_string();
    tm.set_t(0, ThreadStatus::Busy(t1_token.clone()));
    let t1 = tm.get_t(0);
    assert_eq!(t1, Some(ThreadStatus::Busy(t1_token.clone())));
    tm.set_t(0, ThreadStatus::Idle);
    let t2 = tm.get_t(0);
    assert_eq!(t2, Some(ThreadStatus::Idle));
}

#[test]
fn test_get_idle_threads() {
    let mut tm = ThreadsManager::default();
    let max_threads = tm.max_threads;
    let idle_ts = tm.get_idle_ts();
    assert_eq!(idle_ts.len() > 0, true);
    assert_eq!(idle_ts.len() <= max_threads as usize, true);

    let t1_token = "some_token".to_string();
    tm.set_t(0, ThreadStatus::Busy(t1_token.clone()));

    let idle_ts_2 = tm.get_idle_ts();
    assert_eq!(idle_ts_2.len() == idle_ts.len() - 1, true);
}
