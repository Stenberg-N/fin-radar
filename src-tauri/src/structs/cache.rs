use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use log::warn;

use crate::commands::{transactions::Transaction, notes::Note, calendar::CalendarEvent, helpers::create_timestamp};

#[derive(Clone)]
pub enum CacheData {
    Notes(Arc<HashMap<i64, Note>>),
    Transactions(Arc<HashMap<i64, Transaction>>),
    CalendarEvents(Arc<HashMap<i64, CalendarEvent>>),
}
pub enum UpdateTask {
    Delete,
    Update,
}
pub trait AsCacheType<T> {
    fn as_type(&self) -> Option<&T>;
}

pub trait AsCacheTypeMut<T> {
    fn as_type_mut(&mut self) -> Option<&mut T>;
}

impl AsCacheType<Arc<HashMap<i64, Note>>> for CacheData {
    fn as_type(&self) -> Option<&Arc<HashMap<i64, Note>>> {
        match self {
            CacheData::Notes(notes) => Some(notes),
            _ => None,
        }
    }
}

impl AsCacheType<Arc<HashMap<i64, Transaction>>> for CacheData {
    fn as_type(&self) -> Option<&Arc<HashMap<i64, Transaction>>> {
        match self {
            CacheData::Transactions(txs) => Some(txs),
            _ => None,
        }
    }
}

impl AsCacheType<Arc<HashMap<i64, CalendarEvent>>> for CacheData {
    fn as_type(&self) -> Option<&Arc<HashMap<i64, CalendarEvent>>> {
        match self {
            CacheData::CalendarEvents(events) => Some(events),
            _ => None,
        }
    }
}

impl AsCacheTypeMut<Arc<HashMap<i64, Note>>> for CacheData {
    fn as_type_mut(&mut self) -> Option<&mut Arc<HashMap<i64, Note>>> {
        match self {
            CacheData::Notes(notes) => Some(notes),
            _ => None,
        }
    }
}

impl AsCacheTypeMut<Arc<HashMap<i64, Transaction>>> for CacheData {
    fn as_type_mut(&mut self) -> Option<&mut Arc<HashMap<i64, Transaction>>> {
        match self {
            CacheData::Transactions(txs) => Some(txs),
            _ => None,
        }
    }
}

impl AsCacheTypeMut<Arc<HashMap<i64, CalendarEvent>>> for CacheData {
    fn as_type_mut(&mut self) -> Option<&mut Arc<HashMap<i64, CalendarEvent>>> {
        match self {
            CacheData::CalendarEvents(events) => Some(events),
            _ => None,
        }
    }
}

impl From<Vec<Note>> for CacheData {
    fn from(vec: Vec<Note>) -> Self {
        let map = vec.into_iter().map(|n| (n.id, n)).collect();
        CacheData::Notes(Arc::new(map))
    }
}

impl From<Vec<Transaction>> for CacheData {
    fn from(vec: Vec<Transaction>) -> Self {
        let map = vec.into_iter().map(|t| (t.id, t)).collect();
        CacheData::Transactions(Arc::new(map))
    }
}

impl From<Vec<CalendarEvent>> for CacheData {
    fn from(vec: Vec<CalendarEvent>) -> Self {
        let map = vec.into_iter().map(|e| (e.id, e)).collect();
        CacheData::CalendarEvents(Arc::new(map))
    }
}

pub struct Cache {
    cache: Mutex<HashMap<String, CacheData>>
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                cache_guard.clear();
                Ok(())
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string())
            }
        }
    }

    pub fn cache_results(&self, key: String, data: CacheData) -> Result<(), String> {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                cache_guard.insert(key, data);
                Ok(())
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string())
            }
        }
    }

    pub fn update_cache<T>(&self, key: &str, affected: &HashMap<i64, T>, todo: &UpdateTask) -> Result<(), String>
    where 
        CacheData: AsCacheTypeMut<Arc<HashMap<i64, T>>>,
        T: Clone,
    {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                match cache_guard.get_mut(key).and_then(|data| data.as_type_mut()) {
                    Some(data) => {
                        let map = Arc::make_mut(data);
                        match todo {
                            UpdateTask::Delete => {
                                for id in affected.keys() {
                                    map.remove(id);
                                }
                            },
                            UpdateTask::Update => {
                                for (id, item) in affected.iter() {
                                    map.insert(*id, item.clone());
                                }
                            }
                        }
                    },
                    None => {
                        warn!("CACHE UPDATE FAILED ({}): No matching key or type mismatch: {}", create_timestamp(), key);
                    }
                }
                Ok(())
            },
            Err(e) => {
               self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string()) 
            }
        }
    }

    pub fn contains(&self, key: &str) -> Result<bool, String> {
        match self.cache.lock() {
            Ok(cache_guard) => {
                if cache_guard.contains_key(key) {
                    return Ok(true)
                } else {
                    return Ok(false)
                }
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string())
            }
        }
    }

    fn get_cache_data<T>(&self, key: &str) -> Result<Option<T>, String>
    where
        CacheData: AsCacheType<T>,
        T: Clone,
    {
        match self.cache.lock() {
            Ok(cache_guard) => {
                let data = match cache_guard.get(key).and_then(|data| data.as_type()) {
                    Some(cache_data) => Some(cache_data.clone()),
                    _ => None,
                };

                Ok(data)
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string())
            }
        }
    }

    pub fn get_transactions(&self, key: &str) -> Result<Option<Arc<HashMap<i64, Transaction>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Transaction>>>(key)
    }

    pub fn get_notes(&self, key: &str) -> Result<Option<Arc<HashMap<i64, Note>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Note>>>(key)
    }

    pub fn get_calendar_events(&self, key: &str) -> Result<Option<Arc<HashMap<i64, CalendarEvent>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, CalendarEvent>>>(key)
    }
}