use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(unreachable_pub)]
#[derive(FromRow, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Transaction {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) category: String,
    pub(crate) date: String,
    pub(crate) description: String,
    pub(crate) amount: f64,
    pub(crate) _type: String,
}

#[derive(FromRow, Deserialize)]
pub(crate) struct StoredTransaction {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) date: String,
    pub(crate) payload: Vec<u8>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub(crate) struct TransactionPayload {
    pub(crate) category: String,
    pub(crate) description: String,
    pub(crate) amount: f64,
    pub(crate) _type: String,
}

#[derive(Serialize)]
pub(crate) struct TransactionPayloadRef<'a> {
    pub(crate) category: &'a str,
    pub(crate) description: &'a str,
    pub(crate) amount: f64,
    pub(crate) _type: &'a str,
}