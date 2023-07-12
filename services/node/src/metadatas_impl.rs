use crate::defaults::METADATAS_TABLE_NAME;
use crate::error::ServiceError;
use crate::error::ServiceError::RecordNotFound;
use crate::metadatas::{Metadata, MetadataOrdering, MetadataQuery};
use crate::storage_impl::Storage;
use marine_sqlite_connector::{State, Statement, Value};

impl Storage {
    pub fn create_metadatas_table(&self) {
        let table_schema = format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                hash TEXT PRIMARY KEY UNIQUE,
                data_key TEXT not null,
                program_id varchar(255),
                alias varchar(255),
                cid TEXT null,
                public_key TEXT not null,
                version varchar(255) not null,
                loose int check (loose IN (0, 1))
            );",
            METADATAS_TABLE_NAME
        );

        let result = self.connection.execute(table_schema);

        if let Err(error) = result {
            println!("create_transactions_table error: {}", error);
        }
    }

    /**
     * Upon creation of metadata record, it doesnt write metadata CID to the record.
     * Its focusing on creating schema
     */
    pub fn write_metadata(&self, metadata: Metadata) -> Result<(), ServiceError> {
        let s = format!(
            "insert into {} (hash, data_key, program_id, alias, cid, public_key, version, loose) values ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
            METADATAS_TABLE_NAME,
            metadata.hash,
            metadata.data_key,
            metadata.program_id,
            metadata.alias,
            metadata.cid,
            metadata.public_key,
            metadata.version,
            metadata.loose
        );

        let result = self.connection.execute(s);
        match result {
            Ok(_) => return Ok(()),
            Err(error) => {
                log::info!("{:?}", error);
                return Ok(());
            }
        }
    }

    pub fn update_cid(
        &self,
        data_key: String,
        program_id: String,
        alias: String,
        public_key: String,
        cid: String,
        version: String,
    ) -> Result<(), ServiceError> {
        self.connection.execute(format!(
            "
          update {}
          set cid = '{}'
          where data_key = '{}' AND version = '{}' AND program_id = '{}' AND alias = '{}' AND public_key = '{}';
          ",
            METADATAS_TABLE_NAME, cid, data_key, program_id, version, alias, public_key
        ))?;

        Ok(())
    }

    pub fn get_owner_metadata(
        &self,
        data_key: String,
        program_id: String,
        public_key: String,
        alias: String,
        version: String,
    ) -> Result<Metadata, ServiceError> {
        let mut statement = self
            .connection
            .prepare(format!(
                "SELECT * FROM {} WHERE data_key = '{}' AND version = '{}' AND program_id = '{}' AND public_key = '{}' AND alias = '{}'",
                METADATAS_TABLE_NAME, data_key, version, program_id, public_key, alias
            ))
            .unwrap();

        if let State::Row = statement.next()? {
            read(&statement)
        } else {
            Err(RecordNotFound(f!(
                "{data_key}#{program_id}#{public_key}#{alias}#{version}"
            )))
        }
    }

    pub fn get_metadata_by_datakey_and_version(
        &self,
        data_key: String,
        version: String,
    ) -> Result<Vec<Metadata>, ServiceError> {
        let mut statement = self.connection.prepare(f!(
            "SELECT * FROM {METADATAS_TABLE_NAME} WHERE data_key = ? AND version = ?"
        ))?;

        statement.bind(1, &Value::String(data_key.clone()))?;
        statement.bind(2, &Value::String(version.clone()))?;

        let mut metadatas = vec![];

        while let State::Row = statement.next()? {
            metadatas.push(read(&statement)?);
        }

        Ok(metadatas)
    }

    pub fn search_metadatas(
        &self,
        query: Vec<MetadataQuery>,
        ordering: Vec<MetadataOrdering>,
        from: u32,
        to: u32,
    ) -> Result<Vec<Metadata>, ServiceError> {
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
        }
        if to > 0 {
            limit_str = format!("LIMIT {},{}", from, to);
        }

        let s = format!(
            "SELECT * FROM {} {} {} {}",
            METADATAS_TABLE_NAME, query_str, ordering_str, limit_str
        );

        log::info!("{}", s.clone());

        let mut statement = self.connection.prepare(s)?;

        let mut metadatas = Vec::new();

        while let State::Row = statement.next()? {
            metadatas.push(read(&statement)?);
        }

        Ok(metadatas)
    }
}

pub fn read(statement: &Statement) -> Result<Metadata, ServiceError> {
    Ok(Metadata {
        hash: statement.read::<String>(0)?,
        data_key: statement.read::<String>(1)?,
        program_id: statement.read::<String>(2)?,
        alias: statement.read::<String>(3)?,
        cid: statement.read::<String>(4)?,
        public_key: statement.read::<String>(5)?,
        version: statement.read::<String>(6)?,
        loose: statement.read::<i64>(7)? == 1,
    })
}
