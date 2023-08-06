use crate::error::ServiceError;
use crate::storage_impl::{RQLiteResult, Row, Storage};
use crate::{defaults::META_CONTRACT_TABLE_NAME, meta_contract::MetaContract};

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

        let result = Storage::execute(table_schema);
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

        let result = Storage::execute(s);
        Ok(())
    }

    pub fn get_meta_contract_by_id(
        &self,
        program_id: String,
    ) -> Result<MetaContract, ServiceError> {
        let statement = f!("SELECT * FROM {META_CONTRACT_TABLE_NAME} WHERE program_id = ?");

        let result = Storage::read(statement)?;
        let metas = read(result)?;
        match read(result) {
            Ok(metas) => metas
                .first()
                .cloned()
                .ok_or_else(|| ServiceError::RecordNotFound("No record found".to_string())),
            Err(e) => Err(e),
        }
    }
}

pub fn read(result: RQLiteResult) -> Result<Vec<MetaContract>, ServiceError> {
    let mut metas = Vec::new();

    for row in result.rows.unwrap() {
        match row {
            Row::MetaContract(meta_contract) => metas.push(meta_contract),
            _ => {
                return Err(ServiceError::InternalError(format!(
                    "Invalid data format: {}",
                    META_CONTRACT_TABLE_NAME
                )))
            }
        }
    }

    Ok(metas)
}
