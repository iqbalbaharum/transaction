use crate::error::ServiceError;
use crate::error::ServiceError::RecordNotFound;
use crate::storage_impl::Storage;
use crate::{defaults::META_CONTRACT_TABLE_NAME, meta_contract::MetaContract};
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_meta_contract_table(&self) {
        let table_schema = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                program_id varchar(255) not null primary key,
                public_key varchar(255) not null,
                cid varchar(255) not null
            );",
            META_CONTRACT_TABLE_NAME
        );

        let result = self.connection.execute(table_schema);

        if let Err(error) = result {
            println!("create_meta_contract_table error: {}", error);
        }
    }

    /**
     * Upon creation of metadata record, it doesnt write metadata CID to the record.
     * Its focusing on creating schema
     */
    pub fn write_meta_contract(&self, contract: MetaContract) -> Result<(), ServiceError> {
        let s = format!(
            "insert into {} (program_id, public_key, cid) values ('{}', '{}', '{}');",
            META_CONTRACT_TABLE_NAME, contract.program_id, contract.public_key, contract.cid
        );

        self.connection.execute(s)?;

        Ok(())
    }

    pub fn get_meta_contract(&self, program_id: String) -> Result<MetaContract, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT * FROM {META_CONTRACT_TABLE_NAME} WHERE program_id = ?"
        ))?;

        statement.bind(1, &Value::String(program_id.clone()))?;

        if let State::Row = statement.next()? {
            read(&statement)
        } else {
            Err(RecordNotFound(f!("{program_id}")))
        }
    }

    pub fn get_meta_contract_by_id(
        &self,
        program_id: String,
    ) -> Result<MetaContract, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT * FROM {META_CONTRACT_TABLE_NAME} WHERE program_id = ?"
        ))?;

        statement.bind(1, &Value::String(program_id.clone()))?;

        if let State::Row = statement.next()? {
            read(&statement)
        } else {
            Err(RecordNotFound(f!("{program_id}")))
        }
    }
}

pub fn read(statement: &Statement) -> Result<MetaContract, ServiceError> {
    Ok(MetaContract {
        program_id: statement.read::<String>(0)?,
        public_key: statement.read::<String>(1)?,
        cid: statement.read::<String>(2)?,
    })
}
