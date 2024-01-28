use redb::*;

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("data");

pub struct Storage {
    db: redb::Database,
}

impl Default for Storage {
    fn default() -> Self {
        let db_path = match std::env::var("OSINT_GRAPH_DB_PATH") {
            // If the OSINT_GRAPH_DB_PATH environment variable is set, use that.
            Ok(path) => path,

            // Otherwise, use the default path.
            Err(_) => shellexpand::tilde("~/.cache/osint-graph.redb").to_string(),
        };

        Self {
            db: Database::create(db_path).expect("Failed to start DB"),
        }
    }
}

impl Storage {
    #[cfg(test)]
    pub fn test_db() -> Self {
        use redb::backends::InMemoryBackend;
        let db = Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .expect("Failed to start in-memory backend");
        Self { db }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), redb::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let res = table.get(key)?.map(|v| v.value().to_string());
        Ok(res)
    }
}

#[test]
fn test_db_writethrough() {
    let mut storage = Storage::test_db();

    storage.set("test", "test").unwrap();

    assert_eq!(storage.get("test").unwrap(), Some("test".to_string()));
    assert!(storage.get("foo").unwrap().is_none());
}
