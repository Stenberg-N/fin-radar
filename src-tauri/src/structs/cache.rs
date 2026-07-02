use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use log::warn;

use crate::commands::{transactions::Transaction, notes::Note, helpers::create_timestamp};

#[derive(Clone)]
pub enum CacheData {
    Notes(HashMap<i64, Note>),
    Transactions(HashMap<i64, Transaction>),
}

pub enum UpdateTask {
    Delete,
    Update,
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
            Err(e) => Err(e.to_string())
        }
    }

    pub fn cache_results (&self, key: String, data: CacheData) -> Result<(), String> {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                cache_guard.insert(key, data);
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn update_cache (&self, key: &str, affected: &CacheData, todo: &UpdateTask) -> Result<(), String> {
        match self.cache.lock() {
            Ok(mut cache_guard) => {
                match todo {
                    UpdateTask::Delete => {
                        match (cache_guard.get_mut(key), affected) {
                            (Some(CacheData::Transactions(txs)), CacheData::Transactions(affected)) => {
                                let affected_ids: HashSet<i64> = affected.into_iter().map(|t| t.0.to_owned()).collect();
                                for id in affected_ids {
                                    txs.remove(&id);
                                }
                            },
                            (Some(CacheData::Notes(notes)), CacheData::Notes(affected)) => {
                                let affected_ids: HashSet<i64> = affected.into_iter().map(|n| n.0.to_owned()).collect();
                                for id in affected_ids {
                                    notes.remove(&id);
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
                                for (id, transaction) in affected {
                                    txs.insert(*id, transaction.clone());
                                }
                            },
                            (Some(CacheData::Notes(notes)), CacheData::Notes(affected)) => {
                                for (id, note) in affected {
                                    notes.insert(*id, note.clone());
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
            Err(e) => Err(e.to_string())
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
            Err(e) => Err(e.to_string())
        }
    }

    pub fn get_transactions (&self, key: &str) -> Result<Option<HashMap<i64, Transaction>>, String> {
        match self.cache.lock() {
            Ok(cache_guard) => {
                let txs = match cache_guard.get(key) {
                    Some(CacheData::Transactions(txs)) => Some(txs.clone()),
                    _ => None
                };

                Ok(txs)
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn get_notes (&self, key: &str) -> Result<Option<HashMap<i64, Note>>, String> {
        match self.cache.lock() {
            Ok(cache_guard) => {
                let notes = match cache_guard.get(key) {
                    Some(CacheData::Notes(notes)) => Some(notes.clone()),
                    _ => None
                };

                if notes.is_none() {
                    warn!("CACHE FETCH FAILED ({}): No notes in cache for key: {}", create_timestamp(), key);
                }

                Ok(notes)
            },
            Err(e) => Err(e.to_string())
        }
    }
}