use crate::defaults::{STATUS_PENDING, STATUS_SUCCESS, TRANSACTIONS_TABLE_NAME};
use crate::error::ServiceError;
use crate::storage_impl::{RQLiteResult, Row, Storage};
use crate::transaction::Transaction;

impl Storage {
    pub fn create_transactions_table(&self) {
        let table_schema = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                hash TEXT PRIMARY KEY UNIQUE,
                method TEXT NOT NULL,
                program_id TEXT NOT NULL,
                data_key TEXT NOT NULL,
                data TEXT NULL,
                public_key TEXT NOT NULL,
                alias TEXT,
                timestamp INTEGER NOT NULL,
                version varchar(32) NOT NULL,
                mcdata TEXT NULL
            );",
            TRANSACTIONS_TABLE_NAME
        );

        let result = Storage::execute(table_schema);
    }

    pub fn write_transaction(&self, transaction: Transaction) -> Result<String, ServiceError> {
        let s = format!(
            "insert into {} (hash, method, program_id, data_key, data, public_key, alias, timestamp, version, mcdata) values ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
            TRANSACTIONS_TABLE_NAME,
            transaction.hash,
            transaction.method,
            transaction.program_id,
            transaction.data_key,
            transaction.data,
            transaction.public_key,
            transaction.alias,
            transaction.timestamp,
            transaction.version,
            transaction.mcdata
        );

        let result = Storage::execute(s);
        Ok(transaction.hash)
    }

    pub fn get_transaction(&self, hash: String) -> Result<Transaction, ServiceError> {
        let statement = format!(
            "SELECT * FROM {} WHERE hash = {}",
            TRANSACTIONS_TABLE_NAME,
            hash.clone()
        );
        let result = Storage::read(statement)?;
        match read(result) {
            Ok(metas) => metas
                .first()
                .cloned()
                .ok_or_else(|| ServiceError::RecordNotFound("No record found".to_string())),
            Err(e) => Err(e),
        }
    }

    // pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>, ServiceError> {
    //     let statement = format!(
    //         "SELECT * FROM {} WHERE status = {}",
    //         TRANSACTIONS_TABLE_NAME, STATUS_PENDING
    //     );

    //     let result = Storage::read(statement)?;
    //     let txs = read(result)?;
    //     match read(result) {
    //         Ok(txs) => Ok(txs),
    //         Err(e) => Err(e),
    //     }
    // }

    // pub fn get_transactions(
    //     &self,
    //     query: Vec<TransactionQuery>,
    //     ordering: Vec<TransactionOrdering>,
    //     from: u32,
    //     to: u32,
    // ) -> Result<Vec<Transaction>, ServiceError> {
    //     let mut query_str = "".to_string();
    //     let mut ordering_str = "".to_string();
    //     let mut limit_str = "".to_string();

    //     if query.len() > 0 {
    //         let queries: Vec<String> = query
    //             .into_iter()
    //             .map(|param| format!("{} {} '{}'", param.column, param.op, param.query))
    //             .collect();

    //         query_str = format!("WHERE {}", queries.join(" AND "));
    //     }

    //     if ordering.len() > 0 {
    //         let orders: Vec<String> = ordering
    //             .into_iter()
    //             .map(|param| format!("{} {}", param.column, param.sort))
    //             .collect();

    //         ordering_str = format!("ORDER BY {}", orders.join(", "));
    //     } else {
    //         ordering_str = format!("ORDER BY timestamp DESC");
    //     }
    //     if to > 0 {
    //         limit_str = format!("LIMIT {},{}", from, to);
    //     }

    //     let s = format!(
    //         "SELECT * FROM {} {} {} {}",
    //         TRANSACTIONS_TABLE_NAME, query_str, ordering_str, limit_str
    //     );

    //     log::info!("{}", s.clone());

    //     let mut statement = self.connection.prepare(s)?;

    //     let mut transactions = Vec::new();

    //     while let State::Row = statement.next()? {
    //         transactions.push(read(&statement)?);
    //     }

    //     Ok(transactions)
    // }

    // pub fn get_success_transactions(
    //     &self,
    //     from: i64,
    //     to: i64,
    // ) -> Result<Vec<Transaction>, ServiceError> {
    //     let statement = format!(
    //         "SELECT * FROM {} WHERE status = {} AND timestamp BETWEEN {} AND {}",
    //         TRANSACTIONS_TABLE_NAME, STATUS_SUCCESS, from, to
    //     );

    //     let result = Storage::read(statement)?;
    //     let metas = read(result)?;
    //     match read(result) {
    //         Ok(metas) => Ok(metas),
    //         Err(e) => Err(e),
    //     }
    // }
}

pub fn read(result: RQLiteResult) -> Result<Vec<Transaction>, ServiceError> {
    let mut txs = Vec::new();

    for row in result.rows.unwrap() {
        match row {
            Row::Transaction(metadata) => txs.push(metadata),
            _ => {
                return Err(ServiceError::InternalError(format!(
                    "Invalid data format: {}",
                    TRANSACTIONS_TABLE_NAME
                )))
            }
        }
    }

    Ok(txs)
}
