use rusqlite::Connection;
use time::Timespec;

pub struct Database {
    connection: Connection,
}

#[derive(Debug)]
struct Shader {
    rowid: i64, // https://sqlite.org/autoinc.html
    name: String,
    description: String,
    time_created: Timespec,
    source: String,
}

impl Database {
    pub fn new(path: &str) -> Database {
        let connection = Connection::open(path).unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS shader (
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            time_created TEXT NOT NULL,
            source TEXT NOT NULL
        )", &[]).unwrap();
        Database {
            connection: connection
        }
    }

    pub fn list(&self) -> Vec<String> {
        let mut stmt = self.connection.prepare_cached("SELECT ROWID FROM shader").unwrap();
        let iter = stmt.query_map(&[], |row| row.get::<i32, i64>(0).to_string()).unwrap().map(|o| o.unwrap());
        iter.collect()
    }
}
