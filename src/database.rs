use crate::configuration::Configuration;
use rusqlite::{Connection, Result};
use std::path::Path;

#[derive(Debug)]
pub struct Entry {
    pub id: u32,
    pub path: String,
    pub del: bool,
    pub seen: u32,
}

#[derive(Debug)]
pub struct Database<'a> {
    pictures_folder: &'a Path,
    connection: Connection,
}

#[derive(Debug)]
pub struct Stats {
    pub all: u32,
    pub viewed: u32,
    pub not_viewed: u32,
}

impl<'a> Database<'a> {
    pub fn new(config: &Configuration) -> Database {
        let pictures_folder: &String = config.picture_folder.as_ref().unwrap();
        let database_file: &String = config.database_file.as_ref().unwrap();
        let connection: Connection = Connection::open(database_file).unwrap();
        Database {
            pictures_folder: Path::new(pictures_folder),
            connection,
        }
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
