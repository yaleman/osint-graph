use osint_graph_shared::project::Project;
use redb::*;

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("data");
const PROJECT_TABLE: TableDefinition<&str, &str> = TableDefinition::new("projects");

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

    pub fn save_project(&self, project: Project) -> Result<Project, redb::Error> {
        let project = match self.load_project(&project.id.to_string())? {
            Some(_) => project.clone().updated(),
            None => project,
        };

        let write_txn = self.db.begin_write()?;
        {
            let mut table: Table<'_, '_, &str, &str> = write_txn.open_table(PROJECT_TABLE)?;
            let project_id = project.id.to_string();
            let project_value = serde_json::to_string(&project).unwrap();
            table.insert(project_id.as_str(), project_value.as_str())?;
        }
        write_txn.commit()?;
        Ok(project)
    }

    pub fn load_project(&self, id: &str) -> Result<Option<Project>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PROJECT_TABLE)?;

        let res = table.get(id)?.map(|v| v.value().to_string());
        match res {
            Some(val) => {
                let res: Project = serde_json::from_str(&val).unwrap();
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PROJECT_TABLE)?;

        let res = table
            .iter()?
            .map(|row| {
                let (_uuid, row) = row.unwrap();
                serde_json::from_str(&row.value()).unwrap()
            })
            .collect();
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
