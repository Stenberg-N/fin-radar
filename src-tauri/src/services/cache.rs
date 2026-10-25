use std::collections::{HashMap};
use std::sync::{Arc, Mutex, MutexGuard};
use log::warn;

use crate::models::{transaction::Transaction, calendar::CalendarEvent, note::Note};
use crate::utils::time::create_timestamp;

#[allow(unreachable_pub)]
#[derive(Clone)]
pub enum CacheData {
    Notes(Arc<HashMap<i64, Note>>),
    Transactions(Arc<HashMap<i64, Transaction>>),
    CalendarEvents(Arc<HashMap<i64, CalendarEvent>>),
}
pub(crate) enum UpdateTask {
    Delete,
    Update,
}
pub(crate) trait AsCacheType<T> {
    fn as_type(&self) -> Option<&T>;
}

pub(crate) trait AsCacheTypeMut<T> {
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

pub(crate) struct Cache {
    cache: Mutex<HashMap<String, CacheData>>
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self, error_context: &str) -> MutexGuard<'_, HashMap<String, CacheData>> {
        self.cache.lock().unwrap_or_else(|e| {
            let mut guard = e.into_inner();
            guard.clear();
            self.cache.clear_poison();
            warn!("Cache poisoned. Cleared cache and returned to a healthy state.");
            warn!("CACHE POISONED ({}): {}", create_timestamp(), error_context);
            guard
        })
    }

    pub(crate) fn clear(&self, error_context: &str) {
        let mut guard = self.lock(error_context);
        if guard.is_empty() { return; }
        guard.clear();
    }

    pub(crate) fn cache_results(&self, key: String, data: CacheData, error_context: &str) {
        let mut guard = self.lock(error_context);
        guard.insert(key, data);
    }

    pub(crate) fn update_cache<T>(&self, key: &str, affected: &HashMap<i64, T>, todo: &UpdateTask, error_context: &str)
    where 
        CacheData: AsCacheTypeMut<Arc<HashMap<i64, T>>>,
        T: Clone,
    {
        let mut guard = self.lock(error_context);
        match guard.get_mut(key).and_then(|data| data.as_type_mut()) {
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
    }

    pub(crate) fn contains(&self, key: &str) -> Result<bool, String> {
        match self.cache.lock() {
            Ok(cache_guard) => {
                if cache_guard.contains_key(key) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Cleared cache and returned to a healthy state.".to_string())
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
                let data = cache_guard.get(key).and_then(|data| data.as_type()).cloned();
                Ok(data)
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Cleared cache and returned to a healthy state.".to_string())
            }
        }
    }

    pub(crate) fn get_transactions(&self, key: &str) -> Result<Option<Arc<HashMap<i64, Transaction>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Transaction>>>(key)
    }

    pub(crate) fn get_notes(&self, key: &str) -> Result<Option<Arc<HashMap<i64, Note>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Note>>>(key)
    }

    pub(crate) fn get_calendar_events(&self, key: &str) -> Result<Option<Arc<HashMap<i64, CalendarEvent>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, CalendarEvent>>>(key)
    }
}