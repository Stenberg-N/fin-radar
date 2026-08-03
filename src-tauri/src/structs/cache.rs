use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use log::warn;

use crate::commands::{transactions::Transaction, notes::Note, helpers::create_timestamp};

#[derive(Clone)]
pub enum CacheData {
    Notes(Arc<HashMap<i64, Note>>),
    Transactions(Arc<HashMap<i64, Transaction>>),
}
pub enum UpdateTask {
    Delete,
    Update,
}
pub trait AsCacheType<T> {
    fn as_type (&self) -> Option<&T>;
}

impl AsCacheType<Arc<HashMap<i64, Note>>> for CacheData {
    fn as_type (&self) -> Option<&Arc<HashMap<i64, Note>>> {
        match self {
            CacheData::Notes(notes) => Some(notes),
            _ => None,
        }
    }
}

impl AsCacheType<Arc<HashMap<i64, Transaction>>> for CacheData {
    fn as_type (&self) -> Option<&Arc<HashMap<i64, Transaction>>> {
        match self {
            CacheData::Transactions(txs) => Some(txs),
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

pub struct Cache {
    cache: Mutex<HashMap<String, CacheData>>
}

impl Cache {
    pub fn new () -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn clear (&self) -> Result<(), String> {
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

    pub fn cache_results (&self, key: String, data: CacheData) -> Result<(), String> {
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

    pub fn update_cache (&self, key: &str, affected: &CacheData, todo: &UpdateTask) -> Result<(), String> {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                match todo {
                    UpdateTask::Delete => {
                        match (cache_guard.get_mut(key), affected) {
                            (Some(CacheData::Transactions(txs)), CacheData::Transactions(affected)) => {
                                let map = Arc::make_mut(txs);
                                for id in affected.keys() {
                                    map.remove(id);
                                }
                            },
                            (Some(CacheData::Notes(notes)), CacheData::Notes(affected)) => {
                                let map = Arc::make_mut(notes);
                                for id in affected.keys() {
                                    map.remove(id);
                                }
                            },
                            (None, _) => {
                                warn!("CACHE UPDATE FAILED ({}): No matching key: {}", create_timestamp(), key);
                            },
                            _ => {
                                warn!("CACHE UPDATE FAILED ({}): Variant mismatch for key: {}", create_timestamp(), key);
                            }
                        }
                        Ok(())
                    },
                    UpdateTask::Update => {
                        match (cache_guard.get_mut(key), affected) {
                            (Some(CacheData::Transactions(txs)), CacheData::Transactions(affected)) => {
                                let map = Arc::make_mut(txs);
                                for (id, transaction) in affected.iter() {
                                    map.insert(*id, transaction.clone());
                                }
                            },
                            (Some(CacheData::Notes(notes)), CacheData::Notes(affected)) => {
                                let map = Arc::make_mut(notes);
                                for (id, note) in affected.iter() {
                                    map.insert(*id, note.clone());
                                }
                            },
                            (None, _) => {
                                warn!("CACHE UPDATE FAILED ({}): No matching key: {}", create_timestamp(), key);
                            }
                            _ => {
                                warn!("CACHE UPDATE FAILED ({}): Variant mismatch for key: {}", create_timestamp(), key);
                            }
                        }
                        Ok(())
                    }
                }
            },
            Err(e) => {
                self.cache.clear_poison();
                e.into_inner().clear();
                Err("Cache poisoned. Clearing cache.".to_string())
            }
        }
    }

    pub fn contains (&self, key: &str) -> Result<bool, String> {
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

    fn get_cache_data<T> (&self, key: &str, cache_type: &str) -> Result<Option<T>, String>
        where CacheData: AsCacheType<T>, T: Clone,
    {
        match self.cache.lock() {
            Ok(cache_guard) => {
                let data = match cache_guard.get(key) {
                    Some(cache_data) => {
                        match cache_data.as_type() {
                            Some(value) => Some(value.clone()),
                            _ => {
                               warn!("CACHE FETCH FAILED ({}): No {} in cache for key: {}", create_timestamp(), cache_type, key); 
                               None
                            }
                        }
                    },
                    _ => {
                        warn!("CACHE FETCH FAILED ({}): No {} in cache for key: {}", create_timestamp(), cache_type, key); 
                        None
                    }
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

    pub fn get_transactions (&self, key: &str) -> Result<Option<Arc<HashMap<i64, Transaction>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Transaction>>>(key, "transactions")
    }

    pub fn get_notes (&self, key: &str) -> Result<Option<Arc<HashMap<i64, Note>>>, String> {
        self.get_cache_data::<Arc<HashMap<i64, Note>>>(key, "notes")
    }
}