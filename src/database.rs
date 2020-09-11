use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Entry {
    pub id: u32,
    pub path: String,
}

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(file: &Option<String>) -> Database {
        let database_file: &String = file.as_ref().unwrap();
        let connection: Connection = Connection::open(database_file).unwrap();
        Database { connection }
    }
    pub fn init(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE Pics (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            path TEXT NOT NULL,
            del BOOL,
            seen INTEGER
            );",
            params![],
        )?;
        Ok(())
    }
    pub fn get_one(&self) -> Result<Entry> {
        let mut stmt = self.connection.prepare("SELECT id, path FROM Pics WHERE seen = (SELECT MIN(seen) FROM Pics WHERE del IS NOT 1) AND del IS NOT 1 ORDER BY RANDOM() LIMIT 1;")?;
        stmt.query_row(rusqlite::NO_PARAMS, |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })
    }
    pub fn get_seen(&self, id: u32) -> Result<u32> {
        let mut stmt = self
            .connection
            .prepare("SELECT seen FROM Pics WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(row.get(0)?))
    }
    pub fn get_min(&self) -> Result<u32> {
        let mut stmt = self
            .connection
            .prepare("SELECT MIN(seen) FROM Pics WHERE del IS NOT 1 AND seen > 0")?;
        stmt.query_row(rusqlite::NO_PARAMS, |row| Ok(row.get(0)?))
    }
    pub fn get_max(&self) -> Result<u32> {
        let mut stmt = self
            .connection
            .prepare("SELECT MAX(seen) FROM Pics WHERE del IS NOT 1")?;
        stmt.query_row(rusqlite::NO_PARAMS, |row| Ok(row.get(0)?))
    }
    // TODO: this function currently only exists because we only
    // transfer the id of an entry between timer and key_listener
    pub fn get_entry_by_id(&self, id: u32) -> Result<Entry> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, path FROM Pics WHERE id = ?1")?;
        stmt.query_row(params![id], |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })
    }
    // TODO: this function currently only exists because we only
    // have the path when updating the database
    pub fn get_entry_by_path(&self, path: &str) -> Result<Entry> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, path FROM Pics WHERE path = ?1")?;
        stmt.query_row(params![path], |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })
    }
    pub fn add(&self, path: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Pics (path, seen) VALUES (?1, 0)",
            params![path],
        )?;
        Ok(())
    }
    pub fn get_count(&self, seen: u32) -> Result<u32> {
        let mut stmt = self
            .connection
            .prepare("SELECT count(id) FROM Pics WHERE seen = ?1;")?;
        stmt.query_row(params![seen], |row| Ok(row.get(0)?))
    }
    pub fn stats(&self) -> Vec<(u32, u32)> {
        let max = self.get_max().unwrap();
        let mut entries: Vec<(u32, u32)> = Vec::new();
        for x in (0..max + 1).rev() {
            let count = self.get_count(x).unwrap();
            entries.push((x, count));
        }
        entries
    }
    pub fn get_marked(&self) -> Result<Vec<Entry>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, path FROM Pics WHERE del = 1;")?;
        let mut ret: Vec<Entry> = Vec::new();
        let entry_iter = stmt.query_map(rusqlite::NO_PARAMS, |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?;

        for entry in entry_iter {
            ret.push(entry.unwrap());
        }
        Ok(ret)
    }
    pub fn get_all(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.connection.prepare("SELECT id, path FROM Pics;")?;
        let mut ret: Vec<Entry> = Vec::new();
        let entry_iter = stmt.query_map(rusqlite::NO_PARAMS, |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?;

        for entry in entry_iter {
            ret.push(entry.unwrap());
        }
        Ok(ret)
    }
    pub fn remove(&self, entry: &Entry) -> Result<()> {
        self.connection
            .execute("DELETE FROM Pics WHERE id = ?1", params![entry.id])?;
        Ok(())
    }
    pub fn increment(&self, entry: &Entry) -> Result<()> {
        let seen = self.get_seen(entry.id).unwrap();
        if seen != 0 {
            self.connection.execute(
                "UPDATE Pics SET seen=seen + 1 WHERE id = ?1",
                params![entry.id],
            )?;
        } else {
            let min = self.get_min();
            let min = match min {
                Ok(value) => value,
                Err(_e) => 1,
            };
            self.connection.execute(
                "UPDATE Pics SET seen=?1 WHERE id = ?2",
                params![min, entry.id],
            )?;
        }
        Ok(())
    }
    pub fn mark(&self, entry: &Entry) -> Result<()> {
        self.connection
            .execute("UPDATE Pics SET del=1 WHERE id =  ?1", params![entry.id])?;
        Ok(())
    }
}
