// crates/adminx/src/utils/database.rs
use mongodb::Database;
use once_cell::sync::OnceCell;

pub static ADMINX_DATABASE: OnceCell<Database> = OnceCell::new();

pub fn initiate_database(db: Database) {
    ADMINX_DATABASE.set(db).ok(); // ignore error if already set
}

pub fn get_adminx_database() -> &'static Database {
    ADMINX_DATABASE
        .get()
        .expect("ADMINX_DATABASE has not been initialized. Call initiate_database(db) first.")
}
