use crate::defaults::{STATUS_PENDING, STATUS_SUCCESS, TRANSACTIONS_TABLE_NAME};
use crate::error::ServiceError;
use crate::error::ServiceError::InternalError;
use crate::storage_impl::Storage;
use crate::transaction::{Transaction, TransactionOrdering, TransactionQuery};
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_transactions_table(&self) {
        let table_schema = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                hash TEXT PRIMARY KEY UNIQUE,
                method TEXT NOT NULL,
                program_id TEXT NOT NULL,
                data_key TEXT NOT NULL,
                from_peer_id TEXT NOT NULL,
                host_id TEXT NOT NULL,
                status INTEGER NOT NULL,
                data TEXT NULL,
                public_key TEXT NOT NULL,
                alias TEXT,
                timestamp INTEGER NOT NULL,
                error_text TEXT NULL,
                version varchar(32) NOT NULL,
                mcdata TEXT NULL
            );",
            TRANSACTIONS_TABLE_NAME
        );

        let result = self.connection.execute(table_schema);

        if let Err(error) = result {
            println!("create_transactions_table error: {}", error);
        }
    }

    pub fn write_transaction(&self, transaction: Transaction) -> Result<String, ServiceError> {
        let s = format!(
            "insert into {} (hash, method, program_id, data_key, from_peer_id, host_id, status, data, public_key, alias, timestamp, error_text, version, mcdata) values ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
            TRANSACTIONS_TABLE_NAME,
            transaction.hash,
            transaction.method,
            transaction.program_id,
            transaction.data_key,
            transaction.from_peer_id,
            transaction.host_id,
            transaction.status,
            transaction.data,
            transaction.public_key,
            transaction.alias,
            transaction.timestamp,
            transaction.error_text,
            transaction.version,
            transaction.mcdata
        );

        let result = self.connection.execute(s);

        match result {
            Ok(_) => Ok(transaction.hash),
            Err(e) => {
                log::info!("{}", e.to_string());
                Err(InternalError(e.to_string()))
            }
        }
    }

    pub fn update_transaction_status(
        &self,
        hash: String,
        status: i64,
        error_text: String,
    ) -> Result<(), ServiceError> {
        self.connection.execute(format!(
            "
          update {}
          set status = '{}', error_text = '{}'
          where hash = '{}';
          ",
            TRANSACTIONS_TABLE_NAME, status, error_text, hash
        ))?;

        Ok(())
    }

    pub fn get_transaction(&self, hash: String) -> Result<Transaction, ServiceError> {
        let mut statement = self
            .connection
            .prepare(f!("SELECT * FROM {TRANSACTIONS_TABLE_NAME} WHERE hash = ?"))?;

        statement.bind(1, &Value::String(hash.clone()))?;

        if let State::Row = statement.next()? {
            read(&statement)
        } else {
            Err(InternalError(f!(
                "not found non-host records for given key_hash: {hash}"
            )))
        }
    }

    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT * FROM {TRANSACTIONS_TABLE_NAME} WHERE status = ?"
        ))?;

        statement.bind(1, &Value::Integer(STATUS_PENDING))?;

        let mut transactions = Vec::new();

        while let State::Row = statement.next()? {
            transactions.push(read(&statement)?);
        }

        Ok(transactions)
    }

    pub fn get_transactions(
        &self,
        query: Vec<TransactionQuery>,
        ordering: Vec<TransactionOrdering>,
        from: u32,
        to: u32,
    ) -> Result<Vec<Transaction>, ServiceError> {
        let mut query_str = "".to_string();
        let mut ordering_str = "".to_string();
        let mut limit_str = "".to_string();

        if query.len() > 0 {
            let queries: Vec<String> = query
                .into_iter()
                .map(|param| format!("{} {} '{}'", param.column, param.op, param.query))
                .collect();

            query_str = format!("WHERE {}", queries.join(" AND "));
        }

        if ordering.len() > 0 {
            let orders: Vec<String> = ordering
                .into_iter()
                .map(|param| format!("{} {}", param.column, param.sort))
                .collect();

            ordering_str = format!("ORDER BY {}", orders.join(", "));
        } else {
            ordering_str = format!("ORDER BY timestamp DESC");
        }
        if to > 0 {
            limit_str = format!("LIMIT {},{}", from, to);
        }

        let s = format!(
            "SELECT * FROM {} {} {} {}",
            TRANSACTIONS_TABLE_NAME, query_str, ordering_str, limit_str
        );

        log::info!("{}", s.clone());

        let mut statement = self.connection.prepare(s)?;

        let mut transactions = Vec::new();

        while let State::Row = statement.next()? {
            transactions.push(read(&statement)?);
        }

        Ok(transactions)
    }

    pub fn get_success_transactions(
        &self,
        from: i64,
        to: i64,
    ) -> Result<Vec<Transaction>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT * FROM {TRANSACTIONS_TABLE_NAME} WHERE status = ? AND timestamp BETWEEN ? AND ?"
        ))?;

        statement.bind(1, &Value::Integer(STATUS_SUCCESS))?;
        statement.bind(2, &Value::Integer(from))?;
        statement.bind(3, &Value::Integer(to))?;

        let mut transactions = Vec::new();

        while let State::Row = statement.next()? {
            transactions.push(read(&statement)?);
        }

        Ok(transactions)
    }
}

pub fn read(statement: &Statement) -> Result<Transaction, ServiceError> {
    Ok(Transaction {
        hash: statement.read::<String>(0)?,
        method: statement.read::<String>(1)?,
        program_id: statement.read::<String>(2)?,
        data_key: statement.read::<String>(3)?,
        from_peer_id: statement.read::<String>(4)?,
        host_id: statement.read::<String>(5)?,
        status: statement.read::<i64>(6)? as i64,
        data: statement.read::<String>(7)?,
        public_key: statement.read::<String>(8)?,
        alias: statement.read::<String>(9)?,
        timestamp: statement.read::<i64>(10)? as u64,
        error_text: statement.read::<String>(11)?,
        version: statement.read::<String>(12)?,
        mcdata: statement.read::<String>(13)?,
    })
}
