use crate::defaults::{CRON_STATUS_DISABLE, CRON_STATUS_ENABLE};
use crate::error::ServiceError;
use crate::storage_impl::{RQLiteResult, Row};
use crate::{defaults::CRON_TABLE_NAME, storage_impl::Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cron {
    pub program_id: String,
    pub public_key: String,
    pub cid: String,
    pub epoch: i64,
    pub status: i64,
}

impl Storage {
    pub fn create_cron_table(&self) {
        let table_schema = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
            program_id varchar(32) PRIMARY KEY UNIQUE,
            public_key TEXT not null,
            cid varchar(32) not null,
            epoch INTEGER not null,
            status INTEGER not null
            );",
            CRON_TABLE_NAME
        );

        Storage::execute(table_schema);
    }

    /**
     * Creation of cron record
     */
    pub fn write_cron(&self, cron: Cron) -> Result<(), ServiceError> {
        let s = format!(
            "insert into {} (program_id, public_key, cid, epoch, status) values ('{}', '{}', '{}', '{}', '{}');",
            CRON_TABLE_NAME, cron.program_id, cron.public_key, cron.cid, cron.epoch, CRON_STATUS_ENABLE
        );

        Storage::execute(s);

        Ok(())
    }

    pub fn cron_disable(&self, program_id: String) -> Result<(), ServiceError> {
        let s = format!(
            "
          update {}
          set status = '{}'
          where program_id = '{}';
          ",
            CRON_TABLE_NAME, CRON_STATUS_DISABLE, program_id
        );

        Storage::execute(s);

        Ok(())
    }

    pub fn cron_enable(&self, program_id: String) -> Result<(), ServiceError> {
        let s = format!(
            "
          update {}
          set status = '{}'
          where program_id = '{}';
          ",
            CRON_TABLE_NAME, CRON_STATUS_ENABLE, program_id
        );

        Storage::execute(s);

        Ok(())
    }

    pub fn get_cron_by_program_id(&self, program_id: String) -> Result<Cron, ServiceError> {
        let statement = f!("SELECT * FROM {CRON_TABLE_NAME} WHERE program_id = {program_id}");
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

    pub fn get_enabled_crons(&self) -> Result<Vec<Cron>, ServiceError> {
        let statement = f!("SELECT * FROM {CRON_TABLE_NAME} WHERE status = {CRON_STATUS_ENABLE}");

        let result = Storage::read(statement)?;
        let metas = read(result)?;
        match read(result) {
            Ok(metas) => Ok(metas),
            Err(e) => Err(e),
        }
    }

    pub fn get_all_crons(&self) -> Result<Vec<Cron>, ServiceError> {
        let statement = f!("SELECT * FROM {CRON_TABLE_NAME}");
        let result = Storage::read(statement)?;
        let metas = read(result)?;
        match read(result) {
            Ok(crons) => Ok(crons),
            Err(e) => Err(e),
        }
    }
}

pub fn read(result: RQLiteResult) -> Result<Vec<Cron>, ServiceError> {
    let mut crons = Vec::new();

    for row in result.rows.unwrap() {
        match row {
            Row::Cron(cron) => crons.push(cron),
            _ => {
                return Err(ServiceError::InternalError(format!(
                    "Invalid data format: {}",
                    CRON_TABLE_NAME
                )))
            }
        }
    }

    Ok(crons)
}
