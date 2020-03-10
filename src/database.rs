use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Entry {
    pub id: u32,
    pub path: String,
}

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

#[derive(Debug)]
pub struct Stats {
    pub all: u32,
    pub viewed: u32,
    pub not_viewed: u32,
}

impl Database {
    pub fn new(file: &Option<String>) -> Database {
        let database_file: &String = file.as_ref().unwrap();
        let connection: Connection = Connection::open(database_file).unwrap();
        Database {
            connection,
        }
    }
    pub fn get_one(&self) -> Result<Entry> {
        let mut stmt = self.connection.prepare("SELECT id, path FROM Pics WHERE seen = (SELECT MIN(seen) FROM Pics) AND del IS NOT 1 ORDER BY RANDOM() LIMIT 1;")?;
        stmt.query_row(rusqlite::NO_PARAMS, |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })
    }
    pub fn stats(&self) -> Result<Stats> {
        let mut stmt = self.connection.prepare("SELECT count(id) FROM Pics;")?;
        let all = stmt.query_row(rusqlite::NO_PARAMS, |row| row.get::<_, u32>(0))?;
        let mut stmt = self
            .connection
            .prepare("SELECT count(id) FROM Pics WHERE seen IS 0;")?;
        let not_viewed = stmt.query_row(rusqlite::NO_PARAMS, |row| row.get::<_, u32>(0))?;
        let mut stmt = self
            .connection
            .prepare("SELECT count(id) FROM Pics WHERE seen IS NOT 0;")?;
        let viewed = stmt.query_row(rusqlite::NO_PARAMS, |row| row.get::<_, u32>(0))?;
        Ok(Stats {
            all,
            viewed,
            not_viewed,
        })
    }
}
