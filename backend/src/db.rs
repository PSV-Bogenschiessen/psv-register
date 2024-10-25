use diesel::prelude::*;
use diesel::Connection;

pub fn establish_connection(path: &str) -> SqliteConnection {
    SqliteConnection::establish(path).expect("Couldn't connect to database!")
}
